use assert_cmd::Command;

#[test]
fn ai_version_flag_prints_package_version() {
    let mut cmd = Command::cargo_bin("ai").expect("binary should exist");
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("ai 0.3.1"));
}
