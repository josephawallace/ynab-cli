#[test]
fn cli_help_is_available() {
    let output = assert_cmd::Command::cargo_bin("ynab-cli")
        .expect("binary exists")
        .arg("--help")
        .output()
        .expect("CLI starts");
    assert!(output.status.success());
}

#[test]
fn invalid_invocation_has_clap_exit_code() {
    let output = assert_cmd::Command::cargo_bin("ynab-cli")
        .expect("binary exists")
        .arg("transaction")
        .arg("delete")
        .output()
        .expect("CLI starts");
    assert_eq!(output.status.code(), Some(2));
    let error: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("structured invocation error");
    assert_eq!(error["error"]["kind"], "invocation");
}
