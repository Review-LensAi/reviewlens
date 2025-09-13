//! LLM provider abstractions.
//!
//! This module defines the `LlmProvider` trait, which provides a unified
//! interface for interacting with different Large Language Models (LLMs).
//! It ensures that the core engine remains provider-agnostic.

use crate::error::Result;
use async_trait::async_trait;

/// Represents a response from an LLM.
pub struct LlmResponse {
    pub content: String,
    // Could also include metadata like token usage, finish reason, etc.
}

/// A trait for interacting with an LLM provider.
#[async_trait]
pub trait LlmProvider {
    /// Sends a prompt to the LLM and returns the response.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The prompt to send to the model.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `LlmResponse`.
    async fn generate(&self, prompt: &str) -> Result<LlmResponse>;
}

// Example of how you might implement a "null" or "dummy" provider for local-only mode.
pub struct LocalOnlyProvider;

#[async_trait]
impl LlmProvider for LocalOnlyProvider {
    async fn generate(&self, prompt: &str) -> Result<LlmResponse> {
        println!("--- LLM Call (Local-Only Mode) ---");
        println!("Prompt: {}", prompt);
        println!("--- End LLM Call ---");

        Ok(LlmResponse {
            content: "This is a dummy response from the local-only provider.".to_string(),
        })
    }
}
