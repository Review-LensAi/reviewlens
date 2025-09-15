use assert_cmd::Command;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::tempdir;

#[test]
fn check_defaults_to_only_changed() {
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

    // Create initial commit with a secret in one file and a normal file
    fs::write(
        repo.join("unchanged.txt"),
        "api_key = \"ABCDEFGHIJKLMNOPQRSTUVWX\"\n",
    )
    .unwrap();
    fs::write(repo.join("changed.txt"), "hello\n").unwrap();
    StdCommand::new("git")
        .args(["-C", repo_str, "add", "."])
        .output()
        .expect("git add failed");
    StdCommand::new("git")
        .args(["-C", repo_str, "commit", "-m", "init"])
        .output()
        .expect("git commit failed");

    // Modify only one file after the commit
    fs::write(repo.join("changed.txt"), "hello world\n").unwrap();

    let mut cmd = Command::cargo_bin("reviewlens").unwrap();
    let output = cmd
        .args([
            "check",
            "--path",
            repo_str,
            "--base-ref",
            "HEAD",
            "--fail-on",
            "low",
        ])
        .output()
        .expect("failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Reviewed 1 file"));
}
