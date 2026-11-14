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
