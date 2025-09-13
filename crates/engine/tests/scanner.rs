use engine::scanner::{Scanner, TodoScanner, Severity};

#[test]
fn todo_scanner_detects_comment() {
    let scanner = TodoScanner;
    let content = "// TODO: implement";
    let issues = scanner.scan("lib.rs", content).expect("scan should work");
    assert_eq!(issues.len(), 1);
    let issue = &issues[0];
    assert_eq!(issue.file_path, "lib.rs");
    assert_eq!(issue.line_number, 1);
    assert!(matches!(issue.severity, Severity::Info));
}

#[test]
fn todo_scanner_no_issues() {
    let scanner = TodoScanner;
    let issues = scanner.scan("lib.rs", "fn main() {}" ).expect("scan should work");
    assert!(issues.is_empty());
}
