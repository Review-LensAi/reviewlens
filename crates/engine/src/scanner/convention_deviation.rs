//! Scanner to detect deviations from repository conventions.
//!
//! This scanner loads a lightweight index of existing repository code and
//! derives simple pattern-based conventions. New code is scanned for common
//! mismatches such as using `println!` instead of the preferred `log` macros
//! or calling `.unwrap()` for error handling when the repository avoids it.

use crate::config::Config;
use crate::error::Result;
use crate::scanner::{Issue, Scanner};
use regex::Regex;
use serde::Deserialize;
use std::fs;

/// Scanner that flags code which deviates from repository conventions.
pub struct ConventionDeviationScanner;

#[derive(Deserialize)]
struct IndexedDocument {
    filename: String,
    content: String,
}

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
        .all(|d| !d.content.contains("println!") && !d.content.contains("eprintln!"))
    {
        patterns.push(ConventionPattern {
            regex: Regex::new("println!|eprintln!").unwrap(),
            description: "Use logging macros instead of println!/eprintln!",
        });
    }
    if docs.iter().all(|d| !d.content.contains(".unwrap()")) {
        patterns.push(ConventionPattern {
            regex: Regex::new(r"\.unwrap\(\)").unwrap(),
            description: "Avoid unwrap(); use proper error handling",
        });
    }
    if docs.iter().all(|d| !d.content.contains(".expect(")) {
        patterns.push(ConventionPattern {
            regex: Regex::new(r"\.expect\(").unwrap(),
            description: "Avoid expect(); use proper error handling",
        });
    }

    let fn_re = Regex::new(r"fn\s+\w+[^\n]*").unwrap();
    let mut total_fns = 0;
    let mut result_fns = 0;
    for doc in docs {
        for m in fn_re.find_iter(&doc.content) {
            total_fns += 1;
            if m.as_str().contains("->") && m.as_str().contains("Result<") {
                result_fns += 1;
            }
        }
    }
    let require_result = total_fns > 0 && result_fns == total_fns;

    (patterns, require_result)
}

impl Scanner for ConventionDeviationScanner {
    fn name(&self) -> &'static str {
        "Convention Deviation Scanner"
    }

    fn scan(&self, file_path: &str, content: &str, config: &Config) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();
        let index_path = match &config.index_path {
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
                    issues.push(Issue {
                        title: "Convention deviation detected".to_string(),
                        description: pat.description.to_string(),
                        file_path: file_path.to_string(),
                        line_number: i + 1,
                        severity: config.rules.convention_deviation.severity.clone(),
                    });
                    matched = true;
                    break;
                }
            }
            if !matched && require_result {
                let trimmed = line.trim_start();
                if trimmed.starts_with("fn ") && !trimmed.contains("Result<") {
                    issues.push(Issue {
                        title: "Convention deviation detected".to_string(),
                        description: "Functions should return Result<T, E>".to_string(),
                        file_path: file_path.to_string(),
                        line_number: i + 1,
                        severity: config.rules.convention_deviation.severity.clone(),
                    });
                }
            }
        }
        Ok(issues)
    }
}
