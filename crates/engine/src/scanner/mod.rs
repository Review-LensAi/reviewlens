//! Scanners for detecting specific issues like vulnerabilities or secrets.
//!
//! This module defines a `Scanner` trait that can be implemented by different
//! rule-based detectors. This allows for a flexible and extensible scanning system.

use crate::{
    config::{Config, Severity},
    error::Result,
};
use once_cell::sync::Lazy;
use regex::Regex;
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
    pub suggested_fix: Option<String>,
    pub diff: Option<String>,
}

/// A trait for a scanner that checks code for specific issues.
pub trait Scanner: Send + Sync {
    /// Returns the name of the scanner.
    fn name(&self) -> &'static str;

    /// Scans a given file content and returns a list of issues found.
    fn scan(&self, file_path: &str, content: &str, config: &Config) -> Result<Vec<Issue>>;
}

/// Represents an inline suppression directive parsed from source code.
#[derive(Debug, Clone)]
pub struct IgnoreDirective {
    pub rule: String,
    pub reason: Option<String>,
}

/// Mapping of line numbers to suppression directives.
pub type IgnoreMap = HashMap<usize, Vec<IgnoreDirective>>;

static IGNORE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"//\s*reviewlens:ignore\s+([A-Za-z0-9_-]+)(?:\s+(.*))?").unwrap()
});

/// Parses `// reviewlens:ignore` directives within a file's contents.
pub fn parse_ignore_directives(content: &str) -> IgnoreMap {
    let mut map: IgnoreMap = HashMap::new();
    for (i, line) in content.lines().enumerate() {
        if let Some(caps) = IGNORE_REGEX.captures(line) {
            let rule = caps[1].to_string();
            let reason = caps.get(2).map(|m| m.as_str().trim().to_string()).filter(|s| !s.is_empty());
            let target = if line.trim_start().starts_with("//") { i + 2 } else { i + 1 };
            map.entry(target)
                .or_insert_with(Vec::new)
                .push(IgnoreDirective { rule, reason });
        }
    }
    map
}

/// Returns an ignore directive for a given rule at a line, if present.
pub fn find_ignore<'a>(map: &'a IgnoreMap, line: usize, rule: &str) -> Option<&'a IgnoreDirective> {
    map.get(&line)
        .and_then(|vec| vec.iter().find(|d| d.rule == rule))
}

// --- Built-in Scanners ---

pub mod secrets;
pub use secrets::SecretsScanner;
pub mod convention_deviation;
pub use convention_deviation::ConventionDeviationScanner;
pub mod server_xss_go;
pub use server_xss_go::ServerXssGoScanner;

static SQL_INJECTION_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new("(?i)db\\.(query|exec|queryrow)\\s*\\(\\s*fmt\\.Sprintf").unwrap(),
        Regex::new("(?i)db\\.(query|exec|queryrow)\\s*\\(\\s*\"[^\"]*\"\\s*\\+").unwrap(),
        Regex::new("(?i)\"(select|insert|update|delete)[^\"]*\"\\s*\\+").unwrap(),
    ]
});

pub struct SqlInjectionGoScanner;
impl Scanner for SqlInjectionGoScanner {
    fn name(&self) -> &'static str {
        "SQL Injection Scanner (Go)"
    }

    fn scan(&self, file_path: &str, content: &str, config: &Config) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();
        let ignores = parse_ignore_directives(content);
        for (i, line) in content.lines().enumerate() {
            for regex in &*SQL_INJECTION_PATTERNS {
                if regex.is_match(line) {
                    if let Some(ignore) = find_ignore(&ignores, i + 1, "sql-injection-go") {
                        log::info!(
                            "Suppressed sql-injection-go at {}:{}{}",
                            file_path,
                            i + 1,
                            ignore
                                .reason
                                .as_ref()
                                .map(|r| format!(" - {}", r))
                                .unwrap_or_default()
                        );
                    } else {
                        issues.push(Issue {
                            title: "Potential SQL Injection".to_string(),
                            description: "Dynamic SQL query construction detected. Use parameterized queries instead.".to_string(),
                            file_path: file_path.to_string(),
                            line_number: i + 1,
                            severity: config.rules.sql_injection_go.severity.clone(),
                            suggested_fix: Some("Use parameterized queries instead of string concatenation.".to_string()),
                            diff: Some(format!("-{}\n+db.Query(\"...\", params)", line.trim())),
                        });
                    }
                    break;
                }
            }
        }
        Ok(issues)
    }
}

static HTTP_DEFAULT_CLIENT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("(?i)http\\.(Get|Post|Head|Do)\\(").unwrap());
static HTTP_CLIENT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("(?i)&?http\\.Client\\{[^}]*\\}").unwrap());

pub struct HttpTimeoutsGoScanner;
impl Scanner for HttpTimeoutsGoScanner {
    fn name(&self) -> &'static str {
        "HTTP Timeouts Scanner (Go)"
    }

    fn scan(&self, file_path: &str, content: &str, config: &Config) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();
        let ignores = parse_ignore_directives(content);
        for (i, line) in content.lines().enumerate() {
            let uses_default_client = HTTP_DEFAULT_CLIENT_REGEX.is_match(line);
            let client_without_timeout =
                HTTP_CLIENT_REGEX.is_match(line) && !line.contains("Timeout:");
            if uses_default_client || client_without_timeout {
                if let Some(ignore) = find_ignore(&ignores, i + 1, "http-timeouts-go") {
                    log::info!(
                        "Suppressed http-timeouts-go at {}:{}{}",
                        file_path,
                        i + 1,
                        ignore
                            .reason
                            .as_ref()
                            .map(|r| format!(" - {}", r))
                            .unwrap_or_default()
                    );
                } else {
                    issues.push(Issue {
                        title: "HTTP Request Without Timeout".to_string(),
                        description:
                            "HTTP requests should set a timeout to avoid hanging indefinitely."
                                .to_string(),
                        file_path: file_path.to_string(),
                        line_number: i + 1,
                        severity: config.rules.http_timeouts_go.severity.clone(),
                        suggested_fix: Some("Use an http.Client with a Timeout set.".to_string()),
                        diff: Some(if uses_default_client {
                            "-http.Get(url)\n+client := &http.Client{Timeout: 10 * time.Second}\n+client.Get(url)"
                                .to_string()
                        } else {
                            format!(
                                "-{}\n+&http.Client{{Timeout: 10 * time.Second}}",
                                line.trim()
                            )
                        }),
                    });
                }
            }
        }
        Ok(issues)
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
        register_scanner("convention-deviation", || {
            Box::new(ConventionDeviationScanner)
        });
        register_scanner("server-xss-go", || Box::new(ServerXssGoScanner));
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
    if config.rules.convention_deviation.enabled {
        if let Some(factory) = registry.get("convention-deviation") {
            scanners.push(factory());
        }
    }
    if config.rules.server_xss_go.enabled {
        if let Some(factory) = registry.get("server-xss-go") {
            scanners.push(factory());
        }
    }

    scanners
}
