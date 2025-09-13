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
use crate::scanner::Scanner;

/// The main engine struct.
pub struct ReviewEngine {
    config: Config,
    scanners: Vec<Box<dyn Scanner>>,
}

impl ReviewEngine {
    /// Creates a new instance of the review engine from a given configuration.
    pub fn new(config: Config) -> Self {
        let scanners = crate::scanner::load_enabled_scanners(&config);
        Self { config, scanners }
    }

    /// Runs a complete code review analysis on a given diff.
    pub async fn run(&self, diff: &str) -> Result<()> {
        println!("Engine running with config: {:?}", self.config);
        println!("Analyzing diff: {}", diff);
        let loaded: Vec<&str> = self.scanners.iter().map(|s| s.name()).collect();
        println!("Loaded scanners: {:?}", loaded);
        // 1. Parse the diff.
        // 2. Use scanner to find hard-coded issues.
        // 3. Use RAG to fetch relevant context from the codebase.
        // 4. Use LLM to generate insights and suggestions.
        // 5. Generate a report.
        todo!("Implement the main review logic");
    }
}
