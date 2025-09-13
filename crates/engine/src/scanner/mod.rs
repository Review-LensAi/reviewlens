//! Scanners for detecting specific issues like vulnerabilities or secrets.
//!
//! This module defines a `Scanner` trait that can be implemented by different
//! rule-based detectors. This allows for a flexible and extensible scanning system.

use crate::{config::Config, error::Result};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Mutex, Once};

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
pub trait Scanner: Send + Sync {
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
                    description: "A 'TODO' was found, which may indicate unfinished work."
                        .to_string(),
                    file_path: file_path.to_string(),
                    line_number: i + 1,
                    severity: Severity::Info,
                });
            }
        }
        Ok(issues)
    }
}

/// Factory type for creating scanners.
pub type ScannerFactory = fn() -> Box<dyn Scanner>;

/// Global registry of scanners accessible by name.
static REGISTRY: Lazy<Mutex<HashMap<&'static str, ScannerFactory>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Registers a scanner factory under a specific name.
///
/// External crates can call this to add custom scanners.
pub fn register_scanner(name: &'static str, constructor: ScannerFactory) {
    let mut registry = REGISTRY.lock().unwrap();
    registry.insert(name, constructor);
}

fn register_builtin_scanners() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        register_scanner("todo", || Box::new(TodoScanner));
        #[cfg(feature = "owasp_top_5")]
        register_scanner("owasp-top-5", || Box::new(OwaspTop5Scanner));
        #[cfg(feature = "secrets")]
        register_scanner("secrets", || Box::new(SecretsScanner));
    });
}

/// Returns all scanners enabled via configuration.
pub fn load_enabled_scanners(config: &Config) -> Vec<Box<dyn Scanner>> {
    register_builtin_scanners();
    let registry = REGISTRY.lock().unwrap();
    let mut scanners = Vec::new();
    if config.rules.owasp_top_5 {
        if let Some(factory) = registry.get("owasp-top-5") {
            scanners.push(factory());
        }
    }
    if config.rules.secrets {
        if let Some(factory) = registry.get("secrets") {
            scanners.push(factory());
        }
    }
    scanners
}

#[cfg(feature = "owasp_top_5")]
pub struct OwaspTop5Scanner;

#[cfg(feature = "owasp_top_5")]
impl Scanner for OwaspTop5Scanner {
    fn name(&self) -> &'static str {
        "OWASP Top 5 Scanner"
    }

    fn scan(&self, _file_path: &str, _content: &str) -> Result<Vec<Issue>> {
        Ok(Vec::new())
    }
}

#[cfg(feature = "secrets")]
pub struct SecretsScanner;

#[cfg(feature = "secrets")]
impl Scanner for SecretsScanner {
    fn name(&self) -> &'static str {
        "Secrets Scanner"
    }

    fn scan(&self, _file_path: &str, _content: &str) -> Result<Vec<Issue>> {
        Ok(Vec::new())
    }
}
