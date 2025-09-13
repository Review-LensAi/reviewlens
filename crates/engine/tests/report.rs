use engine::config::{Config, Severity};
use engine::report::{
    MarkdownGenerator, ReportGenerator, ReviewReport, RuntimeMetadata, TimingInfo,
};
use engine::scanner::Issue;

#[test]
fn markdown_generator_no_issues() {
    let generator = MarkdownGenerator;
    let report = ReviewReport {
        summary: "All good".into(),
        issues: vec![],
        code_quality: vec![],
        hotspots: vec![],
        mermaid_diagram: None,
        config: Config::default(),
        metadata: RuntimeMetadata {
            ruleset_version: "v1".into(),
            model: Some("test-model".into()),
            driver: "null".into(),
            timings: TimingInfo { total_ms: 0 },
            index_warm: true,
        },
    };
    let md = generator.generate(&report).unwrap();
    assert!(md.contains("✅ No issues found."));
    assert!(md.contains("No code quality issues found."));
    assert!(md.contains("No hotspots identified."));
    assert!(md.contains("\"ruleset_version\": \"v1\""));
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
        suggested_fix: Some("Apply the recommended change".into()),
        diff: Some("-old\n+new".into()),
    };
    let report = ReviewReport {
        summary: "Issues".into(),
        issues: vec![issue],
        code_quality: vec!["Use snake_case for variables".into()],
        hotspots: vec!["src/main.rs:10 - complex function".into()],
        mermaid_diagram: Some("graph TD;A-->B;".into()),
        config: Config::default(),
        metadata: RuntimeMetadata {
            ruleset_version: "v1".into(),
            model: Some("test-model".into()),
            driver: "null".into(),
            timings: TimingInfo { total_ms: 0 },
            index_warm: false,
        },
    };
    let md = generator.generate(&report).unwrap();
    assert!(md.contains("Test issue"));
    assert!(md.contains("lib.rs:42"));
    assert!(md.contains("Apply the recommended change"));
    assert!(md.contains("Diff suggestion for `Test issue` at `lib.rs:42`"));
    assert!(md.contains("-old"));
    assert!(md.contains("Use snake_case for variables"));
    assert!(md.contains("src/main.rs:10 - complex function"));
    assert!(md.contains("```mermaid"));
    assert!(md.contains("A-->B"));
    assert!(md.contains("\"driver\": \"null\""));
}
