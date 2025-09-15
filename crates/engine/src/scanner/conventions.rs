use std::sync::Mutex;

use crate::config::Config;
use crate::error::Result;
use crate::rag::InMemoryVectorStore;
use crate::scanner::{find_ignore, parse_ignore_directives, Issue, Scanner};

#[derive(Default)]
pub struct ConventionsScanner {
    baseline: Mutex<Option<Baseline>>,
}

#[derive(Clone)]
struct Baseline {
    prefers_logging_macros: bool,
    discourage_unwrap: bool,
}

impl ConventionsScanner {
    fn ensure_baseline(&self, config: &Config) -> Option<Baseline> {
        let mut guard = self.baseline.lock().unwrap();
        if guard.is_none() {
            if let Some(path) = config.index_path() {
                if let Ok(store) = InMemoryVectorStore::load_from_disk(path) {
                    let mut log_macro = 0usize;
                    let mut println = 0usize;
                    let mut unwrap_expect = 0usize;
                    let mut result_err = 0usize;
                    for doc in store.documents() {
                        for line in &doc.log_patterns {
                            if line.contains("log::") {
                                log_macro += 1;
                            }
                            if line.contains("println!") || line.contains("eprintln!") {
                                println += 1;
                            }
                        }
                        for line in &doc.error_snippets {
                            if line.contains(".unwrap()") || line.contains(".expect(") {
                                unwrap_expect += 1;
                            }
                            if line.contains("Result<") || line.contains("Err(") {
                                result_err += 1;
                            }
                        }
                    }
                    *guard = Some(Baseline {
                        prefers_logging_macros: log_macro >= println,
                        discourage_unwrap: result_err >= unwrap_expect,
                    });
                }
            }
        }
        guard.clone()
    }
}

impl Scanner for ConventionsScanner {
    fn name(&self) -> &'static str {
        "Convention Deviation Scanner"
    }

    fn scan(&self, file_path: &str, content: &str, config: &Config) -> Result<Vec<Issue>> {
        let baseline = match self.ensure_baseline(config) {
            Some(b) => b,
            None => return Ok(vec![]),
        };

        let mut issues = Vec::new();
        let ignores = parse_ignore_directives(content);
        for (i, line) in content.lines().enumerate() {
            if baseline.prefers_logging_macros
                && (line.contains("println!") || line.contains("eprintln!"))
            {
                if let Some(ignore) = find_ignore(&ignores, i + 1, "conventions") {
                    log::info!(
                        "Suppressed conventions at {}:{}{}",
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
                        title: "Inconsistent Logging".to_string(),
                        description:
                            "Use logging macros (e.g., log::info!) instead of println!/eprintln! per repository conventions."
                                .to_string(),
                        file_path: file_path.to_string(),
                        line_number: i + 1,
                        severity: config.rules.conventions.severity.clone(),
                        suggested_fix: Some("Replace println!/eprintln! with appropriate log:: macros.".to_string()),
                        diff: None,
                    });
                }
            }
            if baseline.discourage_unwrap
                && (line.contains(".unwrap()") || line.contains(".expect("))
            {
                if let Some(ignore) = find_ignore(&ignores, i + 1, "conventions") {
                    log::info!(
                        "Suppressed conventions at {}:{}{}",
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
                        title: "Avoid unwrap/expect".to_string(),
                        description:
                            "Prefer error propagation with Result and ? operator instead of unwrap()/expect() per repository conventions."
                                .to_string(),
                        file_path: file_path.to_string(),
                        line_number: i + 1,
                        severity: config.rules.conventions.severity.clone(),
                        suggested_fix: Some("Propagate errors using ? or handle them explicitly.".to_string()),
                        diff: None,
                    });
                }
            }
        }

        Ok(issues)
    }
}
