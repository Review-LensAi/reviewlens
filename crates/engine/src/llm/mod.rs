//! LLM provider abstractions.
//!
//! This module defines the `LlmProvider` trait, which provides a unified
//! interface for interacting with different Large Language Models (LLMs).
//! It ensures that the core engine remains provider-agnostic.

use crate::config::{LlmConfig, Provider};
use crate::error::{EngineError, Result};
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

pub mod anthropic;
pub mod deepseek;
pub mod openai;

/// Creates an `LlmProvider` instance based on configuration.
pub fn create_llm_provider(config: &LlmConfig) -> Result<Box<dyn LlmProvider>> {
    match config.provider {
        Provider::Openai => {
            let key = config
                .api_key
                .clone()
                .ok_or_else(|| EngineError::Config("Missing OpenAI api_key".into()))?;
            Ok(Box::new(openai::OpenAiProvider::new(
                key,
                config.model.clone(),
                config.temperature,
                config.base_url.clone(),
            )))
        }
        Provider::Anthropic => {
            let key = config
                .api_key
                .clone()
                .ok_or_else(|| EngineError::Config("Missing Anthropic api_key".into()))?;
            Ok(Box::new(anthropic::AnthropicProvider::new(
                key,
                config.model.clone(),
                config.temperature,
                config.base_url.clone(),
            )))
        }
        Provider::Deepseek => {
            let key = config
                .api_key
                .clone()
                .ok_or_else(|| EngineError::Config("Missing DeepSeek api_key".into()))?;
            Ok(Box::new(deepseek::DeepSeekProvider::new(
                key,
                config.model.clone(),
                config.temperature,
                config.base_url.clone(),
            )))
        }
        Provider::Local => Ok(Box::new(LocalOnlyProvider)),
    }
}
