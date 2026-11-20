//! The Podman launcher's flow against a stand-in `podman` (ADR-0016): a script on PATH that
//! records every call. One test in its own binary, because it changes PATH and listens on
//! 127.0.0.1:11434. Skipped when something (a real Ollama) already answers on that port.
#![cfg(unix)]

use ai_workspace_core::config::{PodmanSettings, Runtime};
use ai_workspace_core::runtime::RuntimeManager;
use std::fs;
use std::net::{TcpListener, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// A fake `podman`: logs its arguments, answers `image exists` from `image_present`,
/// `container exists` with "no", and on `run` creates `started` so the test can start
/// listening like the container would.
fn fake_podman(dir: &Path, image_present: bool) {
    let log = dir.join("calls.log");
    let started = dir.join("started");
    let script = format!(
        "#!/bin/sh\n\
         echo \"$*\" >> '{log}'\n\
         case \"$1 $2\" in\n\
           'image exists') exit {image} ;;\n\
           'container exists') exit 1 ;;\n\
         esac\n\
         if [ \"$1\" = run ]; then : > '{started}'; fi\n\
         exit 0\n",
        log = log.display(),
        started = started.display(),
        image = if image_present { 0 } else { 1 },
    );
    let path = dir.join("podman");
    fs::write(&path, script).expect("write the stand-in");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("make it runnable");
}

fn calls(dir: &Path) -> String {
    fs::read_to_string(dir.join("calls.log")).unwrap_or_default()
}

fn temp_dir(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("ai-workspace-podman-{name}-{unique}"));
    fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn set_path(dir: &Path) {
    // SAFETY: this binary holds this one test, so nothing else reads the environment.
    unsafe { std::env::set_var("PATH", dir) };
}

#[test]
fn the_launcher_starts_and_stops_only_its_own_container_and_never_pulls() {
    if TcpStream::connect_timeout(
        &"127.0.0.1:11434".parse().unwrap(),
        Duration::from_millis(200),
    )
    .is_ok()
    {
        eprintln!("skipped: something already answers on 127.0.0.1:11434");
        return;
    }
    let settings = PodmanSettings::default();

    // Podman not installed: an empty PATH.
    let empty = temp_dir("none");
    set_path(&empty);
    let status = RuntimeManager::new()
        .ensure_runtime(Runtime::Podman, &settings)
        .expect("not installed is a status, not an error");
    assert!(!status.running);
    assert!(
        status.detail.contains("Podman is not installed"),
        "{}",
        status.detail
    );

    // The image is missing: reported with the pull command, nothing is run or pulled.
    let missing = temp_dir("missing");
    fake_podman(&missing, false);
    set_path(&missing);
    let status = RuntimeManager::new()
        .ensure_runtime(Runtime::Podman, &settings)
        .expect("a status");
    assert!(!status.running);
    assert!(
        status
            .detail
            .contains("podman pull docker.io/ollama/ollama:latest"),
        "{}",
        status.detail
    );
    let log = calls(&missing);
    assert!(
        !log.lines()
            .any(|l| l.starts_with("run ") || l.starts_with("pull ")),
        "{log}"
    );

    // The image is there: the container is run, then stopped when the manager goes away.
    let ok = temp_dir("ok");
    fake_podman(&ok, true);
    set_path(&ok);
    let marker = ok.join("started");
    let listener = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(20);
        while !marker.exists() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(20));
        }
        // Stand in for Ollama inside the container.
        let socket = TcpListener::bind("127.0.0.1:11434").expect("listen like the container");
        thread::sleep(Duration::from_secs(3));
        drop(socket);
    });
    {
        let mut manager = RuntimeManager::new();
        let status = manager
            .ensure_runtime(Runtime::Podman, &settings)
            .expect("started");
        assert!(status.running && status.started_here, "{status:?}");
        let log = calls(&ok);
        assert!(log.contains("run --detach --rm --pull=never --name ai-workspace-ollama --publish 127.0.0.1:11434:11434"), "{log}");
        assert!(!log.contains("stop"), "not stopped while in use");
    }
    let log = calls(&ok);
    assert!(
        log.lines()
            .last()
            .is_some_and(|l| l == "stop --time 5 ai-workspace-ollama"),
        "stopped when the manager is dropped: {log}"
    );
    assert!(
        !log.lines()
            .any(|l| l.starts_with("rm ") || l.starts_with("pull ")),
        "nothing removed (no leftover) and nothing pulled: {log}"
    );
    listener.join().expect("listener thread");
}
