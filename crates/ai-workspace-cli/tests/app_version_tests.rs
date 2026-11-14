use assert_cmd::Command;

#[test]
fn version_flag_prints_command_name_and_package_version() {
    let mut cmd = Command::cargo_bin("ai-workspace").expect("binary should exist");
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(format!("ai-workspace {}\n", env!("CARGO_PKG_VERSION")));
}

#[test]
fn help_names_the_command_ai_workspace() {
    let mut cmd = Command::cargo_bin("ai-workspace").expect("binary should exist");
    cmd.arg("--help");
    let output = cmd.assert().success().get_output().stdout.clone();
    let help = String::from_utf8(output).expect("help output should be UTF-8");

    for expected in [
        "  ai-workspace init",
        "  ai-workspace search --query TEXT [--project PROJECT] [--limit N]",
        "  ai-workspace --help",
    ] {
        assert!(
            help.lines().any(|line| line.starts_with(expected)),
            "help should contain a line starting with {expected:?}\n{help}"
        );
    }

    // The old name must not survive on any usage line.
    assert!(
        !help
            .lines()
            .any(|line| line == "  ai" || line.starts_with("  ai ")),
        "help still names the command `ai`\n{help}"
    );
}

#[test]
fn usage_errors_name_the_command_ai_workspace() {
    let mut cmd = Command::cargo_bin("ai-workspace").expect("binary should exist");
    cmd.args(["project", "show"]);
    cmd.assert().failure().stderr(predicates::str::contains(
        "usage: ai-workspace project show PROJECT",
    ));
}

#[test]
fn help_into_a_closed_pipe_does_not_panic() {
    // A pipe whose reading end is already closed, like `ai-workspace --help | head`
    // after `head` has exited. Every write to it fails with "broken pipe".
    let (reader, writer) = std::io::pipe().expect("pipe should open");
    drop(reader);

    let output = std::process::Command::new(assert_cmd::cargo::cargo_bin("ai-workspace"))
        .arg("--help")
        .stdout(writer)
        .output()
        .expect("binary should run");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "status {:?}, stderr:\n{stderr}",
        output.status
    );
    assert!(stderr.is_empty(), "expected no stderr, got:\n{stderr}");
}
