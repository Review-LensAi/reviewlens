use engine::config::Config;
use engine::ReviewEngine;

fn diff_for_file(path: &str, line: &str) -> String {
    format!(
        "diff --git a/{0} b/{0}\n--- a/{0}\n+++ b/{0}\n@@ -0,0 +1 @@\n+{1}\n",
        path, line
    )
}

#[tokio::test]
async fn generates_fallback_summary() {
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("secret.txt");
    let content = "api_key = \"ABCDEFGHIJKLMNOP\""; // triggers secret scanner
    std::fs::write(&file_path, content).unwrap();
    let diff = diff_for_file("secret.txt", content);

    let engine = ReviewEngine::new(Config::default()).unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    let report = engine.run(&diff).await.unwrap();

    assert!(report.summary.contains("Reviewed 1 file"));
    assert!(report.summary.contains("Potential Secret Found"));
}
