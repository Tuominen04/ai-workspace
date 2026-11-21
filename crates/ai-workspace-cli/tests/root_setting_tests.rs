//! The workspace folder can be saved with `config set-root` and is found again
//! without `AI_WORKSPACE_HOME`. Every test gives the binary its own HOME and
//! XDG_CONFIG_HOME so nothing touches the real home directory.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

struct Sandbox {
    home: PathBuf,
    config_home: PathBuf,
}

impl Sandbox {
    fn new(name: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let home = std::env::temp_dir().join(format!("ai-workspace-root-{name}-{unique}"));
        let config_home = home.join("xdg");
        fs::create_dir_all(&config_home).expect("sandbox should be created");
        Self { home, config_home }
    }

    fn config_file(&self) -> PathBuf {
        self.config_home.join("ai-workspace").join("config.toml")
    }

    fn command(&self) -> Command {
        let mut cmd = Command::cargo_bin("ai-workspace").expect("binary should exist");
        cmd.env_remove("AI_WORKSPACE_HOME")
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", &self.config_home);
        cmd
    }

    fn folder(&self, name: &str) -> PathBuf {
        let folder = self.home.join(name);
        fs::create_dir_all(&folder).expect("folder should be created");
        fs::canonicalize(folder).expect("folder should canonicalize")
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.home);
    }
}

fn stdout_of(cmd: &mut Command) -> String {
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("output should be UTF-8")
}

fn root_line(output: &str) -> &str {
    output
        .lines()
        .find_map(|line| line.strip_prefix("Root: "))
        .expect("output should have a Root line")
}

#[test]
fn saved_folder_is_used_without_the_environment_variable() {
    let sandbox = Sandbox::new("saved");
    let workspace = sandbox.folder("my-workspace");

    let mut set = sandbox.command();
    set.args(["config", "set-root"]).arg(&workspace);
    let set_output = stdout_of(&mut set);
    assert!(set_output.contains(&format!("Workspace folder set to {}", workspace.display())));
    assert!(
        sandbox.config_file().exists(),
        "config file should be written"
    );

    let status = stdout_of(sandbox.command().arg("status"));
    assert_eq!(root_line(&status), workspace.display().to_string());
    assert!(
        status.contains("Root chosen by: user config file"),
        "status should say where the folder came from:\n{status}"
    );
}

#[test]
fn environment_variable_wins_over_the_saved_folder() {
    let sandbox = Sandbox::new("env-wins");
    let saved = sandbox.folder("saved");
    let from_env = sandbox.folder("from-env");

    sandbox
        .command()
        .args(["config", "set-root"])
        .arg(&saved)
        .assert()
        .success();

    let mut status = sandbox.command();
    status.env("AI_WORKSPACE_HOME", &from_env).arg("status");
    let output = stdout_of(&mut status);
    assert_eq!(root_line(&output), from_env.display().to_string());
    assert!(output.contains("Root chosen by: environment variable AI_WORKSPACE_HOME"));
}

#[test]
fn set_root_warns_when_the_environment_variable_would_override_it() {
    let sandbox = Sandbox::new("warn");
    let saved = sandbox.folder("saved");
    let from_env = sandbox.folder("from-env");

    let mut set = sandbox.command();
    set.env("AI_WORKSPACE_HOME", &from_env)
        .args(["config", "set-root"])
        .arg(&saved);
    let output = stdout_of(&mut set);
    assert!(
        output.contains("takes precedence"),
        "expected a warning:\n{output}"
    );
}

#[test]
fn without_any_setting_the_default_folder_is_used() {
    let sandbox = Sandbox::new("default");

    let status = stdout_of(sandbox.command().arg("status"));
    assert_eq!(
        root_line(&status),
        sandbox.home.join("AI").display().to_string()
    );
    assert!(status.contains("Root chosen by: default location"));
}

#[test]
fn set_root_rejects_a_folder_that_does_not_exist() {
    let sandbox = Sandbox::new("missing");
    let missing = sandbox.home.join("not-there");

    sandbox
        .command()
        .args(["config", "set-root"])
        .arg(&missing)
        .assert()
        .failure()
        .stderr(predicates::str::contains("does not exist"));
    assert!(
        !sandbox.config_file().exists(),
        "a rejected folder must not be saved"
    );
}

#[test]
fn set_root_rejects_a_file() {
    let sandbox = Sandbox::new("file");
    let file = sandbox.home.join("a-file");
    fs::write(&file, "x").expect("file should be written");

    sandbox
        .command()
        .args(["config", "set-root"])
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicates::str::contains("not a directory"));
    assert!(!sandbox.config_file().exists());
}

#[test]
fn set_root_without_a_path_prints_usage() {
    let sandbox = Sandbox::new("usage");

    sandbox
        .command()
        .args(["config", "set-root"])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "usage: ai-workspace config set-root PATH",
        ));
}

#[test]
fn a_relative_saved_folder_is_rejected_with_the_file_named() {
    let sandbox = Sandbox::new("relative");
    write_config(
        &sandbox.config_file(),
        "workspace_root = \"relative/path\"\n",
    );

    sandbox.command().arg("status").assert().failure().stderr(
        predicates::str::contains("config.toml").and(predicates::str::contains("absolute")),
    );
}

#[test]
fn a_broken_config_file_does_not_stop_help_version_or_set_root() {
    let sandbox = Sandbox::new("broken");
    write_config(&sandbox.config_file(), "this is = not [valid toml\n");

    sandbox
        .command()
        .arg("status")
        .assert()
        .failure()
        .stderr(predicates::str::contains("config.toml"));

    sandbox.command().arg("--help").assert().success();
    sandbox.command().arg("--version").assert().success();

    // And set-root repairs it.
    let workspace = sandbox.folder("repaired");
    sandbox
        .command()
        .args(["config", "set-root"])
        .arg(&workspace)
        .assert()
        .success();
    let status = stdout_of(sandbox.command().arg("status"));
    assert_eq!(root_line(&status), workspace.display().to_string());
}

#[test]
fn set_root_does_not_touch_the_workspace_folder() {
    let sandbox = Sandbox::new("untouched");
    let workspace = sandbox.folder("empty-workspace");

    sandbox
        .command()
        .args(["config", "set-root"])
        .arg(&workspace)
        .assert()
        .success();

    let entries = fs::read_dir(&workspace)
        .expect("workspace should be readable")
        .count();
    assert_eq!(entries, 0, "set-root must not create the layout; init does");
}

fn write_config(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().expect("config file has a parent"))
        .expect("config folder should be created");
    fs::write(path, text).expect("config file should be written");
}

#[test]
fn a_folder_without_a_workspace_is_described_not_created() {
    // Issue #55: status describes the folder; other commands say to run init.
    let sandbox = Sandbox::new("no-auto-init");
    let empty = sandbox.folder("empty");

    let status = stdout_of(
        sandbox
            .command()
            .env("AI_WORKSPACE_HOME", &empty)
            .arg("status"),
    );
    assert_eq!(root_line(&status), empty.display().to_string());
    assert!(
        status.contains("No workspace in this folder yet; run `ai-workspace init`"),
        "{status}"
    );

    sandbox
        .command()
        .env("AI_WORKSPACE_HOME", &empty)
        .args(["project", "create", "--name", "Router"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(format!(
            "no workspace in {} yet; run `ai-workspace init` to create it",
            empty.display()
        )));

    let left: Vec<_> = fs::read_dir(&empty).expect("readable").collect();
    assert!(left.is_empty(), "nothing was created: {left:?}");
}
