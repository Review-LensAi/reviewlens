use engine::config::Config;
use engine::ReviewEngine;

fn diff_for_file(path: &str, line: &str) -> String {
    format!(
        "diff --git a/{0} b/{0}\n--- a/{0}\n+++ b/{0}\n@@ -0,0 +1 @@\n+{1}\n",
        path, line
    )
}

#[tokio::test]
async fn respects_allow_patterns() {
    let temp = tempfile::tempdir().unwrap();
    let secret_line = "const API_KEY = \"sk_live_1234567890abcdef1234567890abcdef\";";
    std::fs::write(temp.path().join("included.rs"), secret_line).unwrap();
    std::fs::write(temp.path().join("other.rs"), secret_line).unwrap();

    let diff = format!(
        "{}{}",
        diff_for_file("included.rs", secret_line),
        diff_for_file("other.rs", secret_line)
    );

    let mut config = Config::default();
    config.paths.allow = vec!["included.rs".into()];

    let engine = ReviewEngine::new(config).unwrap();

    std::env::set_current_dir(temp.path()).unwrap();
    let report = engine.run(&diff).await.unwrap();

    assert_eq!(report.issues.len(), 1);
    assert_eq!(report.issues[0].file_path, "included.rs");
}

#[tokio::test]
async fn respects_deny_patterns() {
    let temp = tempfile::tempdir().unwrap();
    let secret_line = "const API_KEY = \"sk_live_1234567890abcdef1234567890abcdef\";";
    std::fs::write(temp.path().join("included.rs"), secret_line).unwrap();
    std::fs::write(temp.path().join("excluded.rs"), secret_line).unwrap();

    let diff = format!(
        "{}{}",
        diff_for_file("included.rs", secret_line),
        diff_for_file("excluded.rs", secret_line)
    );

    let mut config = Config::default();
    config.paths.allow = vec!["*.rs".into()];
    config.paths.deny = vec!["excluded.rs".into()];

    let engine = ReviewEngine::new(config).unwrap();

    std::env::set_current_dir(temp.path()).unwrap();
    let report = engine.run(&diff).await.unwrap();

    assert_eq!(report.issues.len(), 1);
    assert_eq!(report.issues[0].file_path, "included.rs");
}
