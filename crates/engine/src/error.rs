//! Custom error types for the engine crate.

use thiserror::Error;

/// A specialized `Result` type for engine operations.
pub type Result<T> = std::result::Result<T, EngineError>;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("LLM provider error: {0}")]
    LlmProvider(String),

    #[error("Scanner error: {0}")]
    Scanner(String),

    #[error("RAG error: {0}")]
    Rag(String),

    #[error("Diff parsing error: {0}")]
    DiffParser(String),

    #[error("Report generation error: {0}")]
    Report(String),

    #[error("An unknown error occurred")]
    Unknown,
}
