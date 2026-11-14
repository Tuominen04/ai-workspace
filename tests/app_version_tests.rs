use assert_cmd::Command;

#[test]
fn version_flag_prints_command_name_and_package_version() {
    let mut cmd = Command::cargo_bin("local-ai").expect("binary should exist");
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(format!("local-ai {}\n", env!("CARGO_PKG_VERSION")));
}

#[test]
fn help_names_the_command_local_ai() {
    let mut cmd = Command::cargo_bin("local-ai").expect("binary should exist");
    cmd.arg("--help");
    let output = cmd.assert().success().get_output().stdout.clone();
    let help = String::from_utf8(output).expect("help output should be UTF-8");

    for expected in [
        "  local-ai init",
        "  local-ai search --query TEXT [--project PROJECT] [--limit N]",
        "  local-ai --help",
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
fn usage_errors_name_the_command_local_ai() {
    let mut cmd = Command::cargo_bin("local-ai").expect("binary should exist");
    cmd.args(["project", "show"]);
    cmd.assert().failure().stderr(predicates::str::contains(
        "usage: local-ai project show PROJECT",
    ));
}
