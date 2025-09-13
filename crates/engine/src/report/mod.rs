//! Report generation logic.
//!
//! This module takes the analysis results (issues, LLM suggestions, etc.)
//! and formats them into a final report, such as a Markdown file.

use crate::error::Result;
use crate::{config::Config, scanner::Issue};

/// Represents the final, consolidated review findings.
pub struct ReviewReport {
    pub summary: String,
    pub issues: Vec<Issue>,
    /// Notes about code quality or convention deviations.
    pub code_quality: Vec<String>,
    /// Paths or descriptions of files considered hotspots.
    pub hotspots: Vec<String>,
    /// Optional Mermaid sequence diagram showing file interactions.
    pub mermaid_diagram: Option<String>,
    pub config: Config,
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

        md.push_str("## Summary\n\n");
        md.push_str(&report.summary);
        md.push_str("\n\n");

        md.push_str("## 🚨 Security Findings\n\n");

        let mut sorted_issues = report.issues.clone();
        sorted_issues.sort_by(|a, b| b.severity.cmp(&a.severity));

        if sorted_issues.is_empty() {
            md.push_str("✅ No issues found.\n");
        } else {
            md.push_str("| Severity | Title | File:Line | Description | Suggested Fix |\n");
            md.push_str("|---|---|---|---|---|\n");
            for issue in &sorted_issues {
                md.push_str(&format!(
                    "| `{:?}` | {} | `{}:{}` | {} | {} |\n",
                    issue.severity,
                    issue.title,
                    issue.file_path,
                    issue.line_number,
                    issue.description,
                    issue
                        .suggested_fix
                        .clone()
                        .unwrap_or_else(|| "-".to_string())
                ));
            }

            for issue in &sorted_issues {
                if let Some(diff) = &issue.diff {
                    md.push_str(&format!(
                        "\n<details>\n<summary>Diff suggestion for `{}` at `{}:{}`</summary>\n\n```diff\n{}\n```\n</details>\n",
                        issue.title, issue.file_path, issue.line_number, diff
                    ));
                }
            }
        }

        md.push_str("\n## 🧹 Code Quality & Conventions\n\n");
        if report.code_quality.is_empty() {
            md.push_str("No code quality issues found.\n");
        } else {
            md.push_str("| Location | Note |\n|---|---|\n");
            for note in &report.code_quality {
                if let Some((loc, desc)) = note.split_once(" - ") {
                    md.push_str(&format!("| `{}` | {} |\n", loc, desc));
                } else {
                    md.push_str(&format!("| {} | |\n", note));
                }
            }
        }

        md.push_str("\n## 🔥 Hotspots\n\n");
        if report.hotspots.is_empty() {
            md.push_str("No hotspots identified.\n");
        } else {
            md.push_str("| File | Changes |\n|---|---|\n");
            for spot in &report.hotspots {
                if let Some((file, changes)) = spot.split_once(" (") {
                    let changes = changes.trim_end_matches(')');
                    md.push_str(&format!("| `{}` | {} |\n", file, changes));
                } else {
                    md.push_str(&format!("| {} | |\n", spot));
                }
            }
        }

        if let Some(diagram) = &report.mermaid_diagram {
            md.push_str("\n## Diagram\n\n");
            md.push_str("```mermaid\n");
            md.push_str(diagram);
            md.push_str("\n```\n");
        }

        md.push_str("\n---\n\n");
        md.push_str("## Appendix: Configuration Snapshot\n\n");
        md.push_str("This review was run with the following configuration:\n\n");
        md.push_str("```json\n");
        let config_json = serde_json::to_string_pretty(&report.config)
            .map_err(|e| crate::error::EngineError::Report(e.to_string()))?;
        md.push_str(&config_json);
        md.push_str("\n```\n");

        Ok(md)
    }
}
