use engine::config::{Config, Severity};
use engine::report::{MarkdownGenerator, ReportGenerator, ReviewReport};
use engine::scanner::Issue;

#[test]
fn markdown_generator_no_issues() {
    let generator = MarkdownGenerator;
    let report = ReviewReport {
        summary: "All good".into(),
        issues: vec![],
        config: Config::default(),
    };
    let md = generator.generate(&report).unwrap();
    assert!(md.contains("✅ No issues found."));
}

#[test]
fn markdown_generator_with_issues() {
    let generator = MarkdownGenerator;
    let issue = Issue {
        title: "Test issue".into(),
        description: "This is a test".into(),
        file_path: "lib.rs".into(),
        line_number: 42,
        severity: Severity::High,
    };
    let report = ReviewReport {
        summary: "Issues".into(),
        issues: vec![issue],
        config: Config::default(),
    };
    let md = generator.generate(&report).unwrap();
    assert!(md.contains("Test issue"));
    assert!(md.contains("lib.rs:42"));
}
