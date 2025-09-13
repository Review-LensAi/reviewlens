//! Report generation logic.
//!
//! This module takes the analysis results (issues, LLM suggestions, etc.)
//! and formats them into a final report, such as a Markdown file.

use crate::error::Result;
use crate::scanner::Issue;

/// Represents the final, consolidated review findings.
pub struct ReviewReport {
    pub summary: String,
    pub issues: Vec<Issue>,
    // Could also include LLM-generated suggestions, diagrams, etc.
}

/// A trait for generating a report from review findings.
pub trait ReportGenerator {
    /// Generates a report as a string.
    ///
    /// # Arguments
    ///
    /// * `report` - The `ReviewReport` containing all the data to be formatted.
    ///
    /// # Returns
    ///
    /// A `Result` containing the formatted report as a string.
    fn generate(&self, report: &ReviewReport) -> Result<String>;
}

/// A generator for creating Markdown-formatted reports.
pub struct MarkdownGenerator;

impl ReportGenerator for MarkdownGenerator {
    fn generate(&self, report: &ReviewReport) -> Result<String> {
        let mut md = String::new();

        md.push_str("# Code Review Report\n\n");

        md.push_str("## summary\n\n");
        md.push_str(&report.summary);
        md.push_str("\n\n");

        md.push_str("## 🚨 Issues Found\n\n");
        if report.issues.is_empty() {
            md.push_str("✅ No issues found.\n");
        } else {
            for issue in &report.issues {
                md.push_str(&format!(
                    "- **[{:?}] {}**\n  - **File**: `{}:{}`\n  - **Description**: {}\n",
                    issue.severity, issue.title, issue.file_path, issue.line_number, issue.description
                ));
            }
        }

        Ok(md)
    }
}
