//! `ai-workspace project set-model`: the command-line twin of the desktop's model setting.

use assert_cmd::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn ai(root: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::cargo_bin("ai-workspace")
        .expect("binary should exist")
        .env("AI_WORKSPACE_HOME", root)
        .env("HOME", root.join("home"))
        .env("XDG_CONFIG_HOME", root.join("xdg"))
        .args(args)
        .output()
        .expect("binary should run")
}

#[test]
fn set_model_is_saved_shown_and_cleared() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("ai-workspace-model-{unique}"));
    assert!(ai(&root, &["init"]).status.success());
    assert!(
        ai(&root, &["project", "create", "--name", "security"])
            .status
            .success()
    );

    let set = ai(&root, &["project", "set-model", "security", "sec"]);
    assert!(set.status.success());
    assert_eq!(
        String::from_utf8_lossy(&set.stdout),
        "Project security now uses the model sec.\n"
    );
    let shown =
        String::from_utf8_lossy(&ai(&root, &["project", "show", "security"]).stdout).to_string();
    assert!(shown.contains("Model: sec\n"), "{shown}");

    let cleared = ai(&root, &["project", "set-model", "security"]);
    assert!(String::from_utf8_lossy(&cleared.stdout).contains("workspace's default model"));
    let shown =
        String::from_utf8_lossy(&ai(&root, &["project", "show", "security"]).stdout).to_string();
    assert!(shown.contains("Model: workspace default\n"), "{shown}");

    let bad = ai(&root, &["project", "set-model", "security", "two words"]);
    assert!(!bad.status.success());
    assert!(String::from_utf8_lossy(&bad.stderr).contains("invalid model name"));

    let usage = ai(&root, &["project", "set-model"]);
    assert!(
        String::from_utf8_lossy(&usage.stderr)
            .contains("usage: ai-workspace project set-model PROJECT [MODEL]")
    );

    let _ = std::fs::remove_dir_all(&root);
}
