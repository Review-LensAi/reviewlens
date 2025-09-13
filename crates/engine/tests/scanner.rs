use engine::config::Config;
use engine::scanner::{Scanner, SecretsScanner};

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
