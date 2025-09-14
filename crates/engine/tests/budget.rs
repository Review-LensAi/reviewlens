use engine::config::Config;
use engine::ReviewEngine;

fn diff_for_file(path: &str, line: &str) -> String {
    format!(
        "diff --git a/{0} b/{0}\n--- a/{0}\n+++ b/{0}\n@@ -0,0 +1 @@\n+{1}\n",
        path, line
    )
}

#[tokio::test]
async fn ignores_token_budget_with_null_provider() {
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("file.rs");
    let content = "fn main() {}";
    std::fs::write(&file_path, content).unwrap();
    let diff = diff_for_file("file.rs", content);

    let mut config = Config::default();
    config.budget.tokens.max_per_run = Some(0);

    let engine = ReviewEngine::new(config).unwrap();

    std::env::set_current_dir(temp.path()).unwrap();
    engine.run(&diff).await.expect("run should succeed");
}

#[tokio::test]
async fn succeeds_within_token_budget() {
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("file.rs");
    let content = "fn main() {}";
    std::fs::write(&file_path, content).unwrap();
    let diff = diff_for_file("file.rs", content);

    let mut config = Config::default();
    config.budget.tokens.max_per_run = Some(1000);

    let engine = ReviewEngine::new(config).unwrap();

    std::env::set_current_dir(temp.path()).unwrap();
    let report = engine.run(&diff).await.unwrap();
    assert!(report.summary.len() > 0 || report.issues.is_empty());
}
