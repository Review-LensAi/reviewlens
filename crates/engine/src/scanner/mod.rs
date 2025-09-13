//! Scanners for detecting specific issues like vulnerabilities or secrets.
//!
//! This module defines a `Scanner` trait that can be implemented by different
//! rule-based detectors. This allows for a flexible and extensible scanning system.

use crate::error::Result;

/// Represents an issue found by a scanner.
#[derive(Debug)]
pub struct Issue {
    pub title: String,
    pub description: String,
    pub file_path: String,
    pub line_number: usize,
    pub severity: Severity,
}

#[derive(Debug)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// A trait for a scanner that checks code for specific issues.
pub trait Scanner {
    /// Returns the name of the scanner.
    fn name(&self) -> &'static str;

    /// Scans a given file content and returns a list of issues found.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The path to the file being scanned.
    /// * `content` - The content of the file.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `Issue`s.
    fn scan(&self, file_path: &str, content: &str) -> Result<Vec<Issue>>;
}

// Example: A simple scanner for finding "TODO" comments.
pub struct TodoScanner;

impl Scanner for TodoScanner {
    fn name(&self) -> &'static str {
        "TODO Scanner"
    }

    fn scan(&self, file_path: &str, content: &str) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();
        for (i, line) in content.lines().enumerate() {
            if line.contains("TODO") {
                issues.push(Issue {
                    title: "Found 'TODO' comment".to_string(),
                    description: "A 'TODO' was found, which may indicate unfinished work.".to_string(),
                    file_path: file_path.to_string(),
                    line_number: i + 1,
                    severity: Severity::Info,
                });
            }
        }
        Ok(issues)
    }
}
