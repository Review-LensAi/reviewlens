//! Scanners for detecting specific issues like vulnerabilities or secrets.
//!
//! This module defines a `Scanner` trait that can be implemented by different
//! rule-based detectors. This allows for a flexible and extensible scanning system.

use crate::{
    config::{Config, Severity},
    error::Result,
};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Mutex, Once};

/// Represents an issue found by a scanner.
#[derive(Debug, Clone)]
pub struct Issue {
    pub title: String,
    pub description: String,
    pub file_path: String,
    pub line_number: usize,
    pub severity: Severity,
}

/// A trait for a scanner that checks code for specific issues.
pub trait Scanner: Send + Sync {
    /// Returns the name of the scanner.
    fn name(&self) -> &'static str;

    /// Scans a given file content and returns a list of issues found.
    fn scan(&self, file_path: &str, content: &str, config: &Config) -> Result<Vec<Issue>>;
}

// --- Built-in Scanners ---

pub mod secrets;
pub use secrets::SecretsScanner;

pub struct SqlInjectionGoScanner;
impl Scanner for SqlInjectionGoScanner {
    fn name(&self) -> &'static str {
        "SQL Injection Scanner (Go)"
    }
    fn scan(&self, _file_path: &str, _content: &str, _config: &Config) -> Result<Vec<Issue>> {
        // Placeholder implementation
        Ok(Vec::new())
    }
}

pub struct HttpTimeoutsGoScanner;
impl Scanner for HttpTimeoutsGoScanner {
    fn name(&self) -> &'static str {
        "HTTP Timeouts Scanner (Go)"
    }
    fn scan(&self, _file_path: &str, _content: &str, _config: &Config) -> Result<Vec<Issue>> {
        // Placeholder implementation
        Ok(Vec::new())
    }
}

// --- Scanner Registry & Loading ---

/// Factory type for creating scanners.
pub type ScannerFactory = fn() -> Box<dyn Scanner>;

/// Global registry of scanners accessible by name.
static REGISTRY: Lazy<Mutex<HashMap<&'static str, ScannerFactory>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Registers a scanner factory under a specific name.
pub fn register_scanner(name: &'static str, constructor: ScannerFactory) {
    let mut registry = REGISTRY.lock().unwrap();
    registry.insert(name, constructor);
}

fn register_builtin_scanners() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        register_scanner("secrets", || Box::new(SecretsScanner));
        register_scanner("sql-injection-go", || Box::new(SqlInjectionGoScanner));
        register_scanner("http-timeouts-go", || Box::new(HttpTimeoutsGoScanner));
    });
}

/// Returns all scanners enabled via configuration.
pub fn load_enabled_scanners(config: &Config) -> Vec<Box<dyn Scanner>> {
    register_builtin_scanners();
    let registry = REGISTRY.lock().unwrap();
    let mut scanners: Vec<Box<dyn Scanner>> = Vec::new();

    if config.rules.secrets.enabled {
        if let Some(factory) = registry.get("secrets") {
            scanners.push(factory());
        }
    }
    if config.rules.sql_injection_go.enabled {
        if let Some(factory) = registry.get("sql-injection-go") {
            scanners.push(factory());
        }
    }
    if config.rules.http_timeouts_go.enabled {
        if let Some(factory) = registry.get("http-timeouts-go") {
            scanners.push(factory());
        }
    }
    // Note: convention_deviation is for RAG, not a simple scanner, so not loaded here.

    scanners
}
