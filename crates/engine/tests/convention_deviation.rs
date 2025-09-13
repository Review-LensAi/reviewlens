use engine::config::Config;
use engine::scanner::{ConventionDeviationScanner, Scanner};
use serde_json::json;
use std::io::Write;
use tempfile::NamedTempFile;

fn build_index(docs: &[&str]) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("create temp index");
    let data = json!({ "documents": docs });
    file.write_all(data.to_string().as_bytes())
        .expect("write index");
    file
}

#[test]
fn detects_println_usage() {
    let index = build_index(&["fn existing() { log::info!(\"hi\"); }"]);
    let mut config = Config::default();
    config.index_path = Some(index.path().to_str().unwrap().to_string());
    let scanner = ConventionDeviationScanner;
    let content = "fn new() { println!(\"hi\"); }";
    let issues = scanner
        .scan("lib.rs", content, &config)
        .expect("scan should work");
    assert_eq!(issues.len(), 1);
    assert!(issues[0]
        .description
        .contains("logging macros instead of println"));
}

#[test]
fn detects_unwrap_usage() {
    let index = build_index(&["fn existing() -> Result<(), anyhow::Error> { Ok(()) }"]);
    let mut config = Config::default();
    config.index_path = Some(index.path().to_str().unwrap().to_string());
    let scanner = ConventionDeviationScanner;
    let content = "fn new() { let x = option.unwrap(); }";
    let issues = scanner
        .scan("lib.rs", content, &config)
        .expect("scan should work");
    assert_eq!(issues.len(), 1);
    assert!(issues[0].description.contains("Avoid unwrap"));
}

#[test]
fn allows_log_usage() {
    let index = build_index(&["fn existing() { log::info!(\"hi\"); }"]);
    let mut config = Config::default();
    config.index_path = Some(index.path().to_str().unwrap().to_string());
    let scanner = ConventionDeviationScanner;
    let content = "fn new() { log::warn!(\"hi\"); }";
    let issues = scanner
        .scan("lib.rs", content, &config)
        .expect("scan should work");
    assert!(issues.is_empty());
}
