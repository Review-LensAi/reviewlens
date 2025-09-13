use engine::report::{MarkdownGenerator, ReviewReport, ReportGenerator};
use engine::scanner::{Issue, Severity};

#[test]
fn markdown_generator_handles_no_issues() {
    let generator = MarkdownGenerator;
    let report = ReviewReport { summary: "All good".into(), issues: vec![] };
    let output = generator.generate(&report).expect("generation should work");
    assert!(output.contains("No issues found"));
}

#[test]
fn markdown_generator_lists_issues() {
    let generator = MarkdownGenerator;
    let issue = Issue {
        title: "Problem".into(),
        description: "Something went wrong".into(),
        file_path: "src/lib.rs".into(),
        line_number: 42,
        severity: Severity::High,
    };
    let report = ReviewReport { summary: "Issues".into(), issues: vec![issue] };
    let output = generator.generate(&report).expect("generation should work");
    assert!(output.contains("Problem"));
    assert!(output.contains("src/lib.rs:42"));
    assert!(output.contains("Something went wrong"));
}
