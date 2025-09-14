use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command as StdCommand;
use tempfile::tempdir;

#[test]
fn print_config_command_produces_valid_json() {
    let mut cmd = Command::cargo_bin("reviewlens").unwrap();
    let output = cmd
        .arg("print-config")
        .arg("--base-ref")
        .arg("HEAD")
        .output()
        .expect("failed to execute command");

    cmd.assert().success();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let mut base_split = stdout.splitn(2, "Base ref:");
    let json_part = base_split.next().unwrap().trim();
    let base_and_providers = base_split.next().expect("expected base ref in output");
    let json: Value = serde_json::from_str(json_part).expect("stdout should start with valid JSON");
    let mut provider_split = base_and_providers.splitn(2, "Compiled providers:");
    let base_line = provider_split.next().unwrap().trim();
    assert_eq!(base_line, "HEAD");
    if let Some(provider_line) = provider_split.next() {
        assert!(provider_line.contains("null"));
    } else {
        panic!("expected providers list in output");
    }

    assert_eq!(json["llm"]["provider"], "null");
    assert_eq!(json["privacy"]["redaction"]["enabled"], true);
    assert_eq!(json["rules"]["secrets"]["severity"], "high");
}

#[test]
fn version_command_displays_version() {
    let mut cmd = Command::cargo_bin("reviewlens").unwrap();
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

    let mut cmd = Command::cargo_bin("reviewlens").unwrap();
    cmd.args([
        "check",
        "--path",
        repo_str,
        "--base-ref",
        "HEAD",
        "--fail-on",
        "low",
        "--output",
        output_str,
    ]);

    let output = cmd.output().expect("failed to execute command");
    assert!(output.status.success());
    assert!(output_path.exists());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Summary: Reviewed 1 file"));
    assert!(stdout.contains("no issues"));
}

#[test]
fn check_command_reports_issues_and_exit_code() {
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

    // Modify file to introduce a secret
    fs::write(repo.join("file.txt"), "api_key = \"ABCDEFGHIJKLMNOP\"\n").unwrap();

    let mut cmd = Command::cargo_bin("reviewlens").unwrap();
    cmd.args([
        "check",
        "--path",
        repo_str,
        "--base-ref",
        "HEAD",
        "--fail-on",
        "low",
    ]);

    cmd.assert().code(1);
}

#[test]
fn check_command_respects_fail_on_from_config() {
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

    // Modify file to introduce a secret
    fs::write(repo.join("file.txt"), "api_key = \"ABCDEFGHIJKLMNOP\"\n").unwrap();

    // Configure critical fail-on threshold
    fs::write(repo.join("reviewlens.toml"), "fail-on = \"critical\"\n").unwrap();

    let config_path = repo.join("reviewlens.toml");
    let config_str = config_path.to_str().unwrap();

    let mut cmd = Command::cargo_bin("reviewlens").unwrap();
    cmd.args([
        "--config", config_str, "check", "--path", repo_str, "--diff", "HEAD",
    ]);

    cmd.assert().code(0);
}

#[test]
fn check_command_without_upstream_or_diff_errors() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let repo_str = repo.to_str().unwrap();

    // Initialize git repository without remote
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

    let mut cmd = Command::cargo_bin("reviewlens").unwrap();
    cmd.args(["check", "--path", repo_str]);

    cmd.assert().code(2);
}

#[test]
fn check_command_redacts_secrets_in_report() {
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

    // Modify file to introduce a secret
    fs::write(
        repo.join("file.txt"),
        "api_key = \"ABCDEFGHIJKLMNOPQRSTUVWX\"\n",
    )
    .unwrap();

    // Configure redaction pattern to remove the secret value and key
    fs::write(
        repo.join("reviewlens.toml"),
        "[privacy.redaction]\nenabled = true\npatterns = [\"api_key\", \"ABCDEFGHIJKLMNOPQRSTUVWX\"]\n",
    )
    .unwrap();
    let config_path = repo.join("reviewlens.toml");
    let config_str = config_path.to_str().unwrap();

    let output_path = repo.join("review_report.md");
    let output_str = output_path.to_str().unwrap();

    let mut cmd = Command::cargo_bin("reviewlens").unwrap();
    cmd.args([
        "--config", config_str, "check", "--path", repo_str, "--diff", "HEAD", "--output", output_str,
    ]);

    let output = cmd.output().expect("failed to execute command");
    assert_eq!(output.status.code(), Some(1));
    let report = fs::read_to_string(output_path).unwrap();
    assert!(report.contains("[REDACTED]"));
    assert!(!report.contains("api_key"));
    assert!(!report.contains("ABCDEFGHIJKLMNOPQRSTUVWX"));
}

#[test]
fn check_command_generates_json_report_and_redacts_secrets() {
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

    // Modify file to introduce a secret
    fs::write(
        repo.join("file.txt"),
        "api_key = \"ABCDEFGHIJKLMNOPQRSTUVWX\"\n",
    )
    .unwrap();

    // Configure redaction pattern to remove the secret value and key
    fs::write(
        repo.join("reviewlens.toml"),
        "[privacy.redaction]\nenabled = true\npatterns = [\"api_key\", \"ABCDEFGHIJKLMNOPQRSTUVWX\"]\n",
    )
    .unwrap();
    let config_path = repo.join("reviewlens.toml");
    let config_str = config_path.to_str().unwrap();

    let mut cmd = Command::cargo_bin("reviewlens").unwrap();
    cmd.args([
        "--config",
        config_str,
        "check",
        "--path",
        repo_str,
        "--base-ref",
        "HEAD",
        "--fail-on",
        "low",
        "--output",
        output_str,
    ]);

    let output = cmd.output().expect("failed to execute command");
    assert_eq!(output.status.code(), Some(1));

    let report_path = Path::new("review_report.json");
    assert!(report_path.exists());
    let report = fs::read_to_string(report_path).unwrap();
    assert!(report.contains("[REDACTED]"));
    assert!(!report.contains("api_key"));
    assert!(!report.contains("ABCDEFGHIJKLMNOPQRSTUVWX"));
}
