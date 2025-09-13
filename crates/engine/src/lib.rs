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
        println!("Engine running with config: {:?}", self.config);
        println!("Analyzing diff: {}", diff);

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
        for issue in &issues {
            let _ = rag
                .retrieve(&format!(
                    "{}:{} {}",
                    issue.file_path, issue.line_number, issue.description
                ))
                .await?;
        }

        // 4. Call the selected LLM provider for suggestions.
        let prompt = format!(
            "Provide a review summary for the following issues: {:?}",
            issues
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
