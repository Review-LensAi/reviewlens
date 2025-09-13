use engine::config::Config;
use engine::scanner::{ConventionDeviationScanner, Scanner};
use regex::Regex;
use serde_json::json;
use std::io::Write;
use tempfile::NamedTempFile;

fn extract_function_signatures(content: &str) -> Vec<String> {
    let re = Regex::new(r"(?m)^\s*fn\s+\w+[^\n]*").unwrap();
    re.find_iter(content)
        .map(|m| m.as_str().trim().to_string())
        .collect()
}

fn extract_log_patterns(content: &str) -> Vec<String> {
    content
        .lines()
        .filter(|l| l.contains("log::") || l.contains("println!") || l.contains("eprintln!"))
        .map(|l| l.trim().to_string())
        .collect()
}

fn extract_error_snippets(content: &str) -> Vec<String> {
    content
        .lines()
        .filter(|l| {
            l.contains(".unwrap()")
                || l.contains(".expect(")
                || l.contains("Result<")
                || l.contains("Err(")
        })
        .map(|l| l.trim().to_string())
        .collect()
}

fn build_index(docs: &[(&str, &str)]) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("create temp index");
    let documents: Vec<_> = docs
        .iter()
        .map(|(f, c)| {
            json!({
                "filename": f,
                "content": c,
                "function_signatures": extract_function_signatures(c),
                "log_patterns": extract_log_patterns(c),
                "error_snippets": extract_error_snippets(c),
            })
        })
        .collect();
    let data = json!({ "documents": documents });
    file
        .write_all(data.to_string().as_bytes())
        .expect("write index");
    file.flush().expect("flush index");
    file
}

#[test]
fn detects_println_usage() {
    let index = build_index(&[("existing.rs", "fn existing() { log::info!(\"hi\"); }")]);
    let mut config = Config::default();
    config.index_path = Some(index.path().to_str().unwrap().to_string());
    let scanner = ConventionDeviationScanner;
    let content = "fn new() { println!(\"hi\"); }";
    let issues = scanner
        .scan("lib.rs", content, &config)
        .expect("scan should work");
    assert_eq!(issues.len(), 1);
    assert!(issues[0].description.contains("logging macros"));
}

#[test]
fn detects_unwrap_usage() {
    let index = build_index(&[("existing.rs", "fn existing() -> Result<(), anyhow::Error> { Ok(()) }")]);
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
    let index = build_index(&[("existing.rs", "fn existing() { log::info!(\"hi\"); }")]);
    let mut config = Config::default();
    config.index_path = Some(index.path().to_str().unwrap().to_string());
    let scanner = ConventionDeviationScanner;
    let content = "fn new() { log::warn!(\"hi\"); }";
    let issues = scanner
        .scan("lib.rs", content, &config)
        .expect("scan should work");
    assert!(issues.is_empty());
}

#[test]
fn detects_eprintln_usage() {
    let index = build_index(&[("existing.rs", "fn existing() { log::info!(\"hi\"); }")]);
    let mut config = Config::default();
    config.index_path = Some(index.path().to_str().unwrap().to_string());
    let scanner = ConventionDeviationScanner;
    let content = "fn new() { eprintln!(\"hi\"); }";
    let issues = scanner
        .scan("lib.rs", content, &config)
        .expect("scan should work");
    assert_eq!(issues.len(), 1);
    assert!(issues[0].description.contains("logging macros"));
}

#[test]
fn detects_expect_usage() {
    let index = build_index(&[("existing.rs", "fn existing() -> Result<(), anyhow::Error> { Ok(()) }")]);
    let mut config = Config::default();
    config.index_path = Some(index.path().to_str().unwrap().to_string());
    let scanner = ConventionDeviationScanner;
    let content = "fn new() { let x = option.expect(\"hi\"); }";
    let issues = scanner
        .scan("lib.rs", content, &config)
        .expect("scan should work");
    assert_eq!(issues.len(), 1);
    assert!(issues[0].description.contains("Avoid expect"));
}

#[test]
fn detects_missing_result_return() {
    let index = build_index(&[("existing.rs", "fn existing() -> Result<(), anyhow::Error> { Ok(()) }")]);
    let mut config = Config::default();
    config.index_path = Some(index.path().to_str().unwrap().to_string());
    let scanner = ConventionDeviationScanner;
    let content = "fn new() { do_something(); }";
    let issues = scanner
        .scan("lib.rs", content, &config)
        .expect("scan should work");
    assert_eq!(issues.len(), 1);
    assert!(issues[0].description.contains("Functions should return Result"));
}
