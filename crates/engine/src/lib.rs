//! The core engine for the Intelligent Code Review Agent.
//!
//! This crate contains the primary logic for:
//! - Parsing configurations (`config`).
//! - Handling errors (`error`).
//! - Parsing diffs (`diff_parser`).
//! - Interacting with LLM providers (`llm`).
//! - Performing Retrieval-Augmented Generation (`rag`).
//! - Scanning for vulnerabilities and patterns (`scanner`).
//! - Generating reports (`report`).

// Public modules
pub mod config;
pub mod diff_parser;
pub mod error;
pub mod llm;
pub mod rag;
pub mod report;
pub mod scanner;

use crate::config::Config;
use crate::error::{EngineError, Result};
use crate::llm::{create_llm_provider, LlmProvider};
use crate::rag::{InMemoryVectorStore, RagContextRetriever, VectorStore};
use crate::report::ReviewReport;
use crate::scanner::Scanner;
use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Returns the list of LLM providers compiled into this binary.
pub fn compiled_providers() -> Vec<config::Provider> {
    use config::Provider;
    vec![
        Provider::Null,
        Provider::Openai,
        Provider::Anthropic,
        Provider::Deepseek,
    ]
}

/// Placeholder used when redacting sensitive information.
const REDACTION_PLACEHOLDER: &str = "[REDACTED]";

/// Redacts sensitive information from the provided text based on the
/// configured redaction patterns.
pub fn redact_text(config: &Config, text: &str) -> String {
    if !config.privacy.redaction.enabled || config.privacy.redaction.patterns.is_empty() {
        return text.to_string();
    }

    let mut redacted = text.to_string();
    for pattern in &config.privacy.redaction.patterns {
        if let Ok(re) = Regex::new(pattern) {
            redacted = re.replace_all(&redacted, REDACTION_PLACEHOLDER).to_string();
        }
    }
    redacted
}

/// The main engine struct.
pub struct ReviewEngine {
    config: Config,
    scanners: Vec<Box<dyn Scanner>>,
    llm: Box<dyn LlmProvider>,
}

impl ReviewEngine {
    /// Creates a new instance of the review engine from a given configuration.
    pub fn new(config: Config) -> Result<Self> {
        let llm = create_llm_provider(&config)?;
        let scanners = crate::scanner::load_enabled_scanners(&config);
        Ok(Self {
            config,
            scanners,
            llm,
        })
    }

    /// Returns a reference to the engine's configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Runs a complete code review analysis on a given diff.
    pub async fn run(&self, diff: &str) -> Result<ReviewReport> {
        log::info!("Engine running with config: {:?}", self.config);
        log::debug!("Analyzing diff: {}", diff);

        let mut total_tokens_used: u32 = 0;

        // 1. Parse the diff to identify changed files and hunks.
        let changed_files = diff_parser::parse(diff)?;

        // Build globsets for allowed and denied paths.
        let allow_set = build_globset(&self.config.paths.allow)?;
        let deny_set = build_globset(&self.config.paths.deny)?;

        // Filter changed files based on glob patterns.
        let filtered_files: Vec<_> = changed_files
            .into_iter()
            .filter(|file| {
                let path = Path::new(&file.path);
                allow_set.is_match(path) && !deny_set.is_match(path)
            })
            .collect();

        // Track line churn per file; hotspots are computed after scanning.
        let mut churn_counts: HashMap<String, usize> = HashMap::new();
        for file in &filtered_files {
            let mut changes = 0usize;
            for hunk in &file.hunks {
                for line in &hunk.lines {
                    match line {
                        diff_parser::Line::Added(_) | diff_parser::Line::Removed(_) => {
                            changes += 1;
                        }
                        diff_parser::Line::Context(_) => {}
                    }
                }
            }
            churn_counts.insert(file.path.clone(), changes);
        }

        // 2. Run configured scanners on the filtered files, limiting results to diff hunks.
        let mut issues = Vec::new();
        let mut code_quality = Vec::new();
        for file in filtered_files {
            let content = fs::read_to_string(&file.path)?;
            let mut changed_lines = std::collections::HashSet::new();
            for hunk in &file.hunks {
                let mut new_line = hunk.new_start as usize;
                for line in &hunk.lines {
                    match line {
                        diff_parser::Line::Added(_) => {
                            changed_lines.insert(new_line);
                            new_line += 1;
                        }
                        diff_parser::Line::Context(_) => {
                            new_line += 1;
                        }
                        diff_parser::Line::Removed(_) => {}
                    }
                }
            }

            for scanner in &self.scanners {
                let mut found = scanner.scan(&file.path, &content, &self.config)?;
                found.retain(|issue| changed_lines.contains(&issue.line_number));
                if scanner.name() == "Convention Deviation Scanner" {
                    for issue in found {
                        code_quality.push(format!(
                            "{}:{} - {}",
                            issue.file_path, issue.line_number, issue.description
                        ));
                    }
                } else {
                    issues.append(&mut found);
                }
            }
        }

        // Aggregate hotspots using configurable severity and churn weights.
        let mut issue_counts: HashMap<String, usize> = HashMap::new();
        for issue in &issues {
            *issue_counts.entry(issue.file_path.clone()).or_insert(0) += 1;
        }
        let sev_w = self.config.report.hotspot_weights.severity;
        let churn_w = self.config.report.hotspot_weights.churn;
        let mut file_risks: Vec<(String, u32)> = churn_counts
            .into_iter()
            .map(|(path, churn)| {
                let findings = issue_counts.get(&path).copied().unwrap_or(0) as u32;
                let risk = sev_w * findings + churn_w * (churn as u32);
                (path, risk)
            })
            .collect();
        file_risks.sort_by(|a, b| b.1.cmp(&a.1));
        let hotspots: Vec<String> = file_risks
            .into_iter()
            .filter(|(_, risk)| *risk > 0)
            .take(5)
            .map(|(path, risk)| format!("{path} (risk {risk})"))
            .collect();

        // 3. Retrieve RAG context for flagged regions.
        let vector_store: Box<dyn VectorStore + Send + Sync> =
            if let Some(path) = &self.config.index_path {
                match InMemoryVectorStore::load_from_disk(path) {
                    Ok(store) => Box::new(store),
                    Err(e) => {
                        log::warn!("Failed to load vector index from {}: {}", path, e);
                        Box::new(InMemoryVectorStore::default())
                    }
                }
            } else {
                Box::new(InMemoryVectorStore::default())
            };
        let rag = RagContextRetriever::new(vector_store);
        let mut contexts = Vec::new();
        for issue in &issues {
            if let Ok(ctx) = rag
                .retrieve(&format!(
                    "{}:{} {}",
                    issue.file_path, issue.line_number, issue.description
                ))
                .await
            {
                contexts.push(ctx);
            }
        }

        // 4. Call the selected LLM provider for suggestions.
        let mut prompt = String::new();
        if !contexts.is_empty() {
            prompt.push_str("Context:\n");
            prompt.push_str(&contexts.join("\n\n"));
            prompt.push_str("\n\n");
        }
        prompt.push_str(&format!(
            "Provide a review summary for the following issues: {:?}",
            issues
        ));

        // 4. Redact issue descriptions and contexts before calling the LLM.
        let redacted_issues: Vec<String> = issues
            .iter()
            .map(|issue| {
                let redacted_desc = redact_text(&self.config, &issue.description);
                format!(
                    "{}:{} {} - {}",
                    issue.file_path, issue.line_number, issue.title, redacted_desc
                )
            })
            .collect();
        let redacted_contexts: Vec<String> = contexts
            .iter()
            .map(|c| redact_text(&self.config, c))
            .collect();
        let prompt = format!(
            "Provide a review summary for the following issues:\n{}\nContext:\n{}",
            redacted_issues.join("\n"),
            redacted_contexts.join("\n")
        );

        // 5. Call the selected LLM provider for suggestions.
        if let Some(max) = self.config.budget.tokens.max_per_run {
            if total_tokens_used >= max {
                return Err(EngineError::TokenBudgetExceeded {
                    used: total_tokens_used,
                    max,
                });
            }
        }
        let llm_response = self.llm.generate(&prompt).await?;
        total_tokens_used = total_tokens_used.saturating_add(llm_response.token_usage);
        if let Some(max) = self.config.budget.tokens.max_per_run {
            if total_tokens_used > max {
                return Err(EngineError::TokenBudgetExceeded {
                    used: total_tokens_used,
                    max,
                });
            }
        }

        // 6. Build and return the ReviewReport.
        let report = ReviewReport {
            summary: llm_response.content,
            issues,
            code_quality,
            hotspots,
            mermaid_diagram: None,
            config: self.config.clone(),
        };

        Ok(report)
    }
}

fn build_globset(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = Glob::new(pattern).map_err(|e| EngineError::Config(e.to_string()))?;
        builder.add(glob);
    }
    builder
        .build()
        .map_err(|e| EngineError::Config(e.to_string()))
}
