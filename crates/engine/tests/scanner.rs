use engine::config::Config;
use engine::rag::{Document, InMemoryVectorStore};
use engine::scanner::{ConventionsScanner, Scanner, SecretsScanner};

#[test]
fn secrets_scanner_detects_api_key() {
    let scanner = SecretsScanner;
    let content = r#"
        const API_KEY = "sk_live_1234567890abcdef1234567890abcdef";
    "#;
    let config = Config::default();
    let issues = scanner
        .scan("config.js", content, &config)
        .expect("scan should work");
    assert_eq!(issues.len(), 1);
    let issue = &issues[0];
    assert_eq!(issue.title, "Potential Secret Found");
    assert_eq!(issue.file_path, "config.js");
    assert_eq!(issue.line_number, 2);
    assert_eq!(issue.severity, config.rules.secrets.severity);
}

#[test]
fn conventions_scanner_detects_deviation() {
    let mut store = InMemoryVectorStore::default();
    store.push_document(Document {
        filename: "lib.rs".into(),
        content: String::new(),
        embedding: vec![],
        function_signatures: vec![],
        log_patterns: vec!["log::info!(\"hi\")".into()],
        error_snippets: vec!["Result<()>".into()],
        modified: 0,
    });
    let dir = tempfile::tempdir().unwrap();
    let index_path = dir.path().join("index.json.zst");
    store.save_to_disk(&index_path).unwrap();

    let mut config = Config::default();
    config.index = Some(engine::config::IndexConfig {
        path: index_path.to_string_lossy().into(),
    });

    let scanner = ConventionsScanner::default();
    let content = "fn main() { println!(\"hi\"); let _ = foo().unwrap(); }";
    let issues = scanner
        .scan("src/main.rs", content, &config)
        .expect("scan should work");
    assert_eq!(issues.len(), 2);
}

#[test]
fn secrets_scanner_no_issues() {
    let scanner = SecretsScanner;
    let content = "const api_key_name = 'FOO';";
    let config = Config::default();
    let issues = scanner
        .scan("lib.rs", content, &config)
        .expect("scan should work");
    assert!(issues.is_empty());
}

#[test]
fn secrets_scanner_detects_private_key() {
    let scanner = SecretsScanner;
    let content = r#"
-----BEGIN RSA PRIVATE KEY-----
MIIEowIBAAKCAQEAu...
-----END RSA PRIVATE KEY-----
    "#;
    let config = Config::default();
    let issues = scanner
        .scan("key.pem", content, &config)
        .expect("scan should work");
    assert_eq!(issues.len(), 1);
    let issue = &issues[0];
    assert_eq!(issue.line_number, 2);
}

#[test]
fn secrets_scanner_respects_ignore_directive() {
    let scanner = SecretsScanner;
    let content = "const API_KEY = \"sk_live_1234567890abcdef1234567890abcdef\"; // reviewlens:ignore secrets test";
    let config = Config::default();
    let issues = scanner
        .scan("config.js", content, &config)
        .expect("scan should work");
    assert!(issues.is_empty());
}
