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
use crate::error::Result;
use crate::llm::{create_llm_provider, LlmProvider};
use crate::rag::{InMemoryVectorStore, RagContextRetriever};
use crate::report::ReviewReport;
use crate::scanner::{Scanner, TodoScanner};
use std::fs;

/// The main engine struct.
pub struct ReviewEngine {
    config: Config,
    scanners: Vec<Box<dyn Scanner>>,
    llm: Box<dyn LlmProvider>,
}

impl ReviewEngine {
    /// Creates a new instance of the review engine from a given configuration.
    pub fn new(config: Config) -> Result<Self> {
        let llm = create_llm_provider(&config.llm)?;
        let scanners = crate::scanner::load_enabled_scanners(&config);
        Ok(Self { config,scanners, llm })
    }

    /// Runs a complete code review analysis on a given diff.
    pub async fn run(&self, diff: &str) -> Result<ReviewReport> {
        println!("Engine running with config: {:?}", self.config);
        println!("Analyzing diff: {}", diff);

        // 1. Parse the diff to identify changed files and hunks.
        let changed_files = diff_parser::parse(diff)?;

        // 2. Run configured scanners on the changed files.
        let mut issues = Vec::new();
        let scanners: Vec<Box<dyn Scanner>> = vec![Box::new(TodoScanner)];
        for file in changed_files {
            let content = fs::read_to_string(&file.path)?;
            for scanner in &scanners {
                let mut found = scanner.scan(&file.path, &content)?;
                issues.append(&mut found);
            }
        }

        // 3. Retrieve RAG context for flagged regions.
        let rag = RagContextRetriever::new(Box::new(InMemoryVectorStore::default()));
        for issue in &issues {
            let _ = rag
                .retrieve(&format!("{}:{} {}", issue.file_path, issue.line_number, issue.description))
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
        };

        Ok(report)
    }
}
