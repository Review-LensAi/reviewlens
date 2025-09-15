use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::tempdir;

#[test]
fn check_ci_produces_json_logs() {
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

    let mut cmd = Command::cargo_bin("reviewlens").unwrap();
    let output = cmd
        .env("RUST_LOG", "info")
        .args([
            "check",
            "--path",
            repo_str,
            "--base-ref",
            "HEAD",
            "--ci",
            "--fail-on",
            "low",
            "--output",
            output_str,
        ])
        .output()
        .expect("failed to execute command");

    assert!(output.status.success());
    assert!(output_path.exists());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let mut count = 0;
    for line in stdout.lines().filter(|l| l.trim_start().starts_with('{')) {
        let v: Value = serde_json::from_str(line).expect("log line is valid JSON");
        assert!(v.get("level").is_some());
        assert!(v.get("msg").is_some());
        assert!(v.get("module").is_some());
        assert!(v.get("ts").is_some());
        count += 1;
    }
    assert!(count > 0, "expected at least one JSON log line");
}
