use assert_cmd::Command;
use serde_json::Value;

#[test]
fn print_config_command_produces_valid_json() {
    let mut cmd = Command::cargo_bin("reviewer-cli").unwrap();
    let output = cmd.arg("print-config").output().expect("failed to execute command");

    cmd.assert().success();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON");

    assert_eq!(json["llm"]["provider"], "null");
    assert_eq!(json["privacy"]["redaction"]["enabled"], true);
    assert_eq!(json["rules"]["secrets"]["severity"], "medium");
}
