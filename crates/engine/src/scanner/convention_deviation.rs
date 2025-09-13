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
struct IndexStore {
    documents: Vec<String>,
}

struct ConventionPattern {
    regex: Regex,
    description: &'static str,
}

fn derive_patterns(docs: &[String]) -> Vec<ConventionPattern> {
    let mut patterns = Vec::new();
    if docs.iter().all(|d| !d.contains("println!")) {
        patterns.push(ConventionPattern {
            regex: Regex::new("println!").unwrap(),
            description: "Use logging macros instead of println!",
        });
    }
    if docs.iter().all(|d| !d.contains(".unwrap()")) {
        patterns.push(ConventionPattern {
            regex: Regex::new(r"\.unwrap\(\)").unwrap(),
            description: "Avoid unwrap(); use proper error handling",
        });
    }
    patterns
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
        let patterns = derive_patterns(&store.documents);
        for (i, line) in content.lines().enumerate() {
            for pat in &patterns {
                if pat.regex.is_match(line) {
                    issues.push(Issue {
                        title: "Convention deviation detected".to_string(),
                        description: pat.description.to_string(),
                        file_path: file_path.to_string(),
                        line_number: i + 1,
                        severity: config.rules.convention_deviation.severity.clone(),
                    });
                    break;
                }
            }
        }
        Ok(issues)
    }
}
