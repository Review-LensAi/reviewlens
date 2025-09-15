use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::tempdir;

#[test]
fn ci_requires_model_when_provider_not_null() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let repo_str = repo.to_str().unwrap();

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

    // Config with OpenAI provider but missing model
    fs::write(
        repo.join("reviewlens.toml"),
        "[llm]\nprovider = \"openai\"\napi-key = \"dummy\"\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("reviewlens").unwrap();
    cmd.current_dir(repo);
    cmd.args([
        "--config",
        "reviewlens.toml",
        "check",
        "--ci",
        "--path",
        repo_str,
        "--diff",
        "HEAD",
    ]);

    cmd.assert().code(2);
}

#[test]
fn ci_sets_generation_temperature_to_zero() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let repo_str = repo.to_str().unwrap();

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

    fs::write(
        repo.join("reviewlens.toml"),
        "[generation]\ntemperature = 0.5\n",
    )
    .unwrap();

    let output_path = repo.join("report.json");
    let output_str = output_path.to_str().unwrap();

    let mut cmd = Command::cargo_bin("reviewlens").unwrap();
    cmd.current_dir(repo);
    cmd.args([
        "--config",
        "reviewlens.toml",
        "check",
        "--ci",
        "--path",
        repo_str,
        "--diff",
        "HEAD",
        "--format",
        "json",
        "--output",
        output_str,
        "--no-progress",
    ]);

    cmd.assert().success();
    let report: Value = serde_json::from_str(&fs::read_to_string(output_path).unwrap()).unwrap();
    assert_eq!(report["config"]["generation"]["temperature"], 0.0);
}
