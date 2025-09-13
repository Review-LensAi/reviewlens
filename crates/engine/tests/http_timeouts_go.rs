use engine::config::Config;
use engine::scanner::{HttpTimeoutsGoScanner, Scanner};

#[test]
fn detects_http_get_without_timeout() {
    let scanner = HttpTimeoutsGoScanner;
    let content = r#"
        resp, _ := http.Get("http://example.com")
    "#;
    let config = Config::default();
    let issues = scanner
        .scan("net.go", content, &config)
        .expect("scan should work");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].line_number, 2);
}

#[test]
fn allows_client_with_timeout() {
    let scanner = HttpTimeoutsGoScanner;
    let content = r#"
        client := http.Client{Timeout: 10 * time.Second}
        resp, _ := client.Get("http://example.com")
    "#;
    let config = Config::default();
    let issues = scanner
        .scan("net.go", content, &config)
        .expect("scan should work");
    assert!(issues.is_empty());
}
