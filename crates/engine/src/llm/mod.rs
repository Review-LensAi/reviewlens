//! LLM provider abstractions.
//!
//! This module defines the `LlmProvider` trait, which provides a unified
//! interface for interacting with different Large Language Models (LLMs).
//! It ensures that the core engine remains provider-agnostic.

use crate::config::{Config, Provider};
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

/// The "null" provider for local-only/offline mode.
pub struct NullProvider;

#[async_trait]
impl LlmProvider for NullProvider {
    async fn generate(&self, prompt: &str) -> Result<LlmResponse> {
        log::info!("--- LLM Call (Null Provider) ---");
        log::debug!("Prompt: {}", prompt);
        log::info!("--- End LLM Call ---");

        Ok(LlmResponse {
            content: "This is a dummy response from the null provider.".to_string(),
        })
    }
}

pub mod anthropic;
pub mod deepseek;
pub mod openai;

/// Creates an `LlmProvider` instance based on configuration.
pub fn create_llm_provider(config: &Config) -> Result<Box<dyn LlmProvider>> {
    match &config.llm.provider {
        Provider::Openai => {
            let api_key = config
                .llm
                .api_key
                .clone()
                .ok_or_else(|| EngineError::Config("Missing OpenAI api_key".into()))?;
            let model = config
                .llm
                .model
                .clone()
                .ok_or_else(|| EngineError::Config("Missing model for OpenAI provider".into()))?;
            let temperature = config.generation.temperature.unwrap_or(0.1);
            Ok(Box::new(openai::OpenAiProvider::new(
                api_key,
                model,
                temperature,
                config.llm.base_url.clone(),
            )))
        }
        Provider::Anthropic => {
            let api_key = config
                .llm
                .api_key
                .clone()
                .ok_or_else(|| EngineError::Config("Missing Anthropic api_key".into()))?;
            let model = config
                .llm
                .model
                .clone()
                .ok_or_else(|| {
                    EngineError::Config("Missing model for Anthropic provider".into())
                })?;
            let temperature = config.generation.temperature.unwrap_or(0.1);
            Ok(Box::new(anthropic::AnthropicProvider::new(
                api_key,
                model,
                temperature,
                config.llm.base_url.clone(),
            )))
        }
        Provider::Deepseek => {
            let api_key = config
                .llm
                .api_key
                .clone()
                .ok_or_else(|| EngineError::Config("Missing DeepSeek api_key".into()))?;
            let model = config
                .llm
                .model
                .clone()
                .ok_or_else(|| EngineError::Config("Missing model for DeepSeek provider".into()))?;
            let temperature = config.generation.temperature.unwrap_or(0.1);
            Ok(Box::new(deepseek::DeepSeekProvider::new(
                api_key,
                model,
                temperature,
                config.llm.base_url.clone(),
            )))
        }
        Provider::Null => Ok(Box::new(NullProvider)),
    }
}
