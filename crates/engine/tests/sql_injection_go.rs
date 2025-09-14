use engine::config::{Config, RuleConfig, RulesConfig, Severity};
use engine::scanner::{Scanner, SqlInjectionGoScanner};

/// Build a configuration fixture with only the SQL injection rule enabled.
///
/// Using `Config::default()` pulled in all rules which could introduce
/// nondeterministic behaviour if other tests mutate global configuration.
/// By constructing the relevant rule explicitly we keep this test isolated
/// and deterministic.
fn test_config() -> Config {
    Config {
        rules: RulesConfig {
            sql_injection_go: RuleConfig {
                enabled: true,
                severity: Severity::Medium,
            },
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn detects_dynamic_sql_concatenation() {
    let scanner = SqlInjectionGoScanner;
    let content = r#"
        query := "SELECT * FROM users WHERE name = '" + user + "'"
        rows, _ := db.Query(query)
    "#;
    let config = test_config();
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
    let config = test_config();
    let issues = scanner
        .scan("user.go", content, &config)
        .expect("scan should work");
    assert!(issues.is_empty());
}
