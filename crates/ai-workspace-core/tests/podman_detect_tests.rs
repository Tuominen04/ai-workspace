//! When an Ollama already answers on 127.0.0.1:11434, the session says which one (ADR-0016):
//! the app's container, or another Ollama. One test in its own binary, because it changes PATH
//! and listens on the port; skipped when something real already answers there.
#![cfg(unix)]

use ai_workspace_core::config::{PodmanSettings, Runtime};
use ai_workspace_core::runtime::RuntimeManager;
use std::fs;
use std::net::{TcpListener, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// A stand-in `podman` whose `inspect` reports the app's container as running or not.
fn fake_podman(dir: &Path, ours_running: bool) {
    let script = format!(
        "#!/bin/sh\nif [ \"$1\" = inspect ]; then echo {}; exit 0; fi\nexit 0\n",
        if ours_running { "true" } else { "false" }
    );
    let path = dir.join("podman");
    fs::write(&path, script).expect("write the stand-in");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("runnable");
}

fn dir(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("ai-workspace-detect-{name}-{unique}"));
    fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn detail(runtime: Runtime) -> String {
    let status = RuntimeManager::new()
        .ensure_runtime(runtime, &PodmanSettings::default())
        .expect("a status");
    assert!(status.running && !status.started_here, "{status:?}");
    status.detail
}

#[test]
fn an_ollama_already_running_is_named() {
    if TcpStream::connect_timeout(
        &"127.0.0.1:11434".parse().unwrap(),
        Duration::from_millis(200),
    )
    .is_ok()
    {
        eprintln!("skipped: something already answers on 127.0.0.1:11434");
        return;
    }
    let _ollama = TcpListener::bind("127.0.0.1:11434").expect("stand in for an Ollama");

    // The app's container answers (started by hand, or left from before).
    let ours = dir("ours");
    fake_podman(&ours, true);
    // SAFETY: this binary holds this one test, so nothing else reads the environment.
    unsafe { std::env::set_var("PATH", &ours) };
    for runtime in [Runtime::Podman, Runtime::Ollama] {
        let text = detail(runtime);
        assert!(
            text.contains("in the ai-workspace-ollama container"),
            "{text}"
        );
    }

    // Something else answers while Podman is chosen: said so, with what to do.
    let other = dir("other");
    fake_podman(&other, false);
    unsafe { std::env::set_var("PATH", &other) };
    let text = detail(Runtime::Podman);
    assert!(
        text.contains("not the app's Podman container") && text.contains("stop it"),
        "{text}"
    );
    assert_eq!(
        detail(Runtime::Ollama),
        "Ollama is already running on 127.0.0.1:11434"
    );

    // No Podman at all: not the container.
    let none = dir("none");
    unsafe { std::env::set_var("PATH", &none) };
    assert!(detail(Runtime::Podman).contains("not the app's Podman container"));
}
