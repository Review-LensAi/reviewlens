use engine::{
    config::{Config, TelemetryConfig},
    ReviewEngine,
};

fn diff_for_file(path: &str, line: &str) -> String {
    format!(
        "diff --git a/{0} b/{0}\n--- a/{0}\n+++ b/{0}\n@@ -0,0 +1 @@\n+{1}\n",
        path, line
    )
}

#[tokio::test]
async fn writes_telemetry_events() {
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("secret.txt");
    let content = "api_key = \"ABCDEFGHIJKLMNOP\""; // triggers secret scanner
    std::fs::write(&file_path, content).unwrap();
    let diff = diff_for_file("secret.txt", content);

    let telemetry_path = temp.path().join("telemetry.jsonl");
    let mut config = Config::default();
    config.telemetry = TelemetryConfig {
        enabled: true,
        file: Some(telemetry_path.to_string_lossy().into()),
    };

    let engine = ReviewEngine::new(config).unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    let _ = engine.run(&diff).await.unwrap();

    let data = std::fs::read_to_string(&telemetry_path).unwrap();
    let lines: Vec<&str> = data.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].contains("run_started"));
    assert!(lines[1].contains("finding"));
    assert!(lines[2].contains("run_finished"));
}
