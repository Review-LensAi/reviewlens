use engine::config::Config;
use engine::scanner::{Scanner, SqlInjectionGoScanner};

#[test]
fn detects_dynamic_sql_concatenation() {
    let scanner = SqlInjectionGoScanner;
    let content = r#"
        query := "SELECT * FROM users WHERE name = '" + user + "'"
        rows, _ := db.Query(query)
    "#;
    let config = Config::default();
    let issues = scanner
        .scan("user.go", content, &config)
        .expect("scan should work");
    assert_eq!(issues.len(), 1);
    let issue = &issues[0];
    assert_eq!(issue.line_number, 2);
    assert_eq!(issue.severity, config.rules.sql_injection_go.severity);
}

#[test]
fn allows_parameterized_query() {
    let scanner = SqlInjectionGoScanner;
    let content = r#"
        rows, _ := db.Query("SELECT * FROM users WHERE id = ?", id)
    "#;
    let config = Config::default();
    let issues = scanner
        .scan("user.go", content, &config)
        .expect("scan should work");
    assert!(issues.is_empty());
}
