use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::tempdir;

#[test]
fn print_config_command_produces_valid_json() {
    let mut cmd = Command::cargo_bin("reviewer-cli").unwrap();
    let output = cmd
        .arg("print-config")
        .output()
        .expect("failed to execute command");

    cmd.assert().success();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON");

    assert_eq!(json["llm"]["provider"], "null");
    assert_eq!(json["privacy"]["redaction"]["enabled"], true);
    assert_eq!(json["rules"]["secrets"]["severity"], "medium");
}

#[test]
fn version_command_displays_version() {
    let mut cmd = Command::cargo_bin("reviewer-cli").unwrap();
    let output = cmd
        .arg("version")
        .output()
        .expect("failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), env!("CARGO_PKG_VERSION"));
}

#[test]
fn check_command_respects_path_argument() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let repo_str = repo.to_str().unwrap();

    // Initialize git repository
    StdCommand::new("git")
        .args(["init", repo_str])
        .output()
        .expect("git init failed");
    StdCommand::new("git")
        .args(["-C", repo_str, "config", "user.email", "you@example.com"])
        .output()
        .expect("git config email failed");
    StdCommand::new("git")
        .args(["-C", repo_str, "config", "user.name", "Your Name"])
        .output()
        .expect("git config name failed");

    // Create initial commit
    fs::write(repo.join("file.txt"), "hello\n").unwrap();
    StdCommand::new("git")
        .args(["-C", repo_str, "add", "."])
        .output()
        .expect("git add failed");
    StdCommand::new("git")
        .args(["-C", repo_str, "commit", "-m", "init"])
        .output()
        .expect("git commit failed");

    // Modify file to create diff
    fs::write(repo.join("file.txt"), "hello world\n").unwrap();

    let output_path = repo.join("out.md");
    let output_str = output_path.to_str().unwrap();

    let mut cmd = Command::cargo_bin("reviewer-cli").unwrap();
    cmd.args([
        "check", "--path", repo_str, "--diff", "HEAD", "--output", output_str,
    ]);

    cmd.assert().success();
    assert!(output_path.exists());
}
