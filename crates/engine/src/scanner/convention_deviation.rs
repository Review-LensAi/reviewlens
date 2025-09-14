//! Scanner to detect deviations from repository conventions.
//!
//! This scanner loads a lightweight index of existing repository code and
//! derives simple pattern-based conventions. New code is scanned for common
//! mismatches such as using `println!` instead of the preferred `log` macros
//! or calling `.unwrap()` for error handling when the repository avoids it.

use crate::config::Config;
use crate::error::Result;
use crate::rag::Document as IndexedDocument;
use crate::scanner::{find_ignore, parse_ignore_directives, Issue, Scanner};
use regex::Regex;
use serde::Deserialize;
use std::fs;

/// Scanner that flags code which deviates from repository conventions.
pub struct ConventionDeviationScanner;

#[derive(Deserialize)]
struct IndexStore {
    documents: Vec<IndexedDocument>,
}

struct ConventionPattern {
    regex: Regex,
    description: &'static str,
}

fn derive_patterns(docs: &[IndexedDocument]) -> (Vec<ConventionPattern>, bool) {
    let mut patterns = Vec::new();
    if docs
        .iter()
        .all(|d| d
            .log_patterns
            .iter()
            .all(|l| !l.contains("println!") && !l.contains("eprintln!")))
    {
        patterns.push(ConventionPattern {
            regex: Regex::new("println!|eprintln!").unwrap(),
            description: "Use logging macros instead of println!/eprintln!",
        });
    }
    if docs
        .iter()
        .all(|d| d.error_snippets.iter().all(|l| !l.contains(".unwrap()")))
    {
        patterns.push(ConventionPattern {
            regex: Regex::new(r"\.unwrap\(\)").unwrap(),
            description: "Avoid unwrap(); use proper error handling",
        });
    }
    if docs
        .iter()
        .all(|d| d.error_snippets.iter().all(|l| !l.contains(".expect(")))
    {
        patterns.push(ConventionPattern {
            regex: Regex::new(r"\.expect\(").unwrap(),
            description: "Avoid expect(); use proper error handling",
        });
    }

    let total_fns: usize = docs.iter().map(|d| d.function_signatures.len()).sum();
    let result_fns: usize = docs
        .iter()
        .flat_map(|d| d.function_signatures.iter())
        .filter(|sig| sig.contains("->") && sig.contains("Result<"))
        .count();
    let require_result = total_fns > 0 && result_fns == total_fns;

    (patterns, require_result)
}

impl Scanner for ConventionDeviationScanner {
    fn name(&self) -> &'static str {
        "Convention Deviation Scanner"
    }

    fn scan(&self, file_path: &str, content: &str, config: &Config) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();
        let ignores = parse_ignore_directives(content);
        let index_path = match config.index_path() {
            Some(p) => p,
            None => return Ok(issues),
        };
        let data = match fs::read_to_string(index_path) {
            Ok(d) => d,
            Err(_) => return Ok(issues),
        };
        let store: IndexStore = match serde_json::from_str(&data) {
            Ok(s) => s,
            Err(_) => return Ok(issues),
        };
        let (patterns, require_result) = derive_patterns(&store.documents);
        for (i, line) in content.lines().enumerate() {
            let mut matched = false;
            for pat in &patterns {
                if pat.regex.is_match(line) {
                    if let Some(ignore) = find_ignore(&ignores, i + 1, "convention-deviation") {
                        log::info!(
                            "Suppressed convention-deviation at {}:{}{}",
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
                            title: "Convention deviation detected".to_string(),
                            description: pat.description.to_string(),
                            file_path: file_path.to_string(),
                            line_number: i + 1,
                            severity: config.rules.convention_deviation.severity.clone(),
                            suggested_fix: Some(pat.description.to_string()),
                            diff: Some(format!("-{}\n+// {}", line.trim(), pat.description)),
                        });
                    }
                    matched = true;
                    break;
                }
            }
            if !matched && require_result {
                let trimmed = line.trim_start();
                if trimmed.starts_with("fn ") && !trimmed.contains("Result<") {
                    if let Some(ignore) = find_ignore(&ignores, i + 1, "convention-deviation") {
                        log::info!(
                            "Suppressed convention-deviation at {}:{}{}",
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
                            title: "Convention deviation detected".to_string(),
                            description: "Functions should return Result<T, E>".to_string(),
                            file_path: file_path.to_string(),
                            line_number: i + 1,
                            severity: config.rules.convention_deviation.severity.clone(),
                            suggested_fix: Some(
                                "Update function signature to return Result<T, E>".to_string(),
                            ),
                            diff: Some(format!(
                                "-{}\n+{} -> Result<_, _>",
                                line.trim(),
                                line.trim()
                            )),
                        });
                    }
                }
            }
        }
        Ok(issues)
    }
}
