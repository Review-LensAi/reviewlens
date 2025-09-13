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
use crate::rag::{InMemoryVectorStore, RagContextRetriever};
use crate::report::ReviewReport;
use crate::scanner::Scanner;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::fs;
use std::path::Path;
use regex::Regex;

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
            redacted = re
                .replace_all(&redacted, REDACTION_PLACEHOLDER)
                .to_string();
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

    /// Runs a complete code review analysis on a given diff.
    pub async fn run(&self, diff: &str) -> Result<ReviewReport> {
        log::info!("Engine running with config: {:?}", self.config);
        log::debug!("Analyzing diff: {}", diff);

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

        // 2. Run configured scanners on the filtered files.
        let mut issues = Vec::new();
        for file in filtered_files {
            let content = fs::read_to_string(&file.path)?;
            for scanner in &self.scanners {
                let mut found = scanner.scan(&file.path, &content, &self.config)?;
                issues.append(&mut found);
            }
        }

        // 3. Retrieve RAG context for flagged regions.
        let rag = RagContextRetriever::new(Box::new(InMemoryVectorStore::default()));
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
      
            let context = rag
                .retrieve(&format!("{}:{} {}", issue.file_path, issue.line_number, issue.description))
                .await?;
            contexts.push(context);
        }

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

        let llm_response = self.llm.generate(&prompt).await?;

        // 5. Build and return the ReviewReport.
        let report = ReviewReport {
            summary: llm_response.content,
            issues,
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
