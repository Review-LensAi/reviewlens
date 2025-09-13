use super::{LlmProvider, LlmResponse};
use crate::error::{EngineError, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    model: String,
    temperature: f32,
    base_url: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String, temperature: f32, base_url: Option<String>) -> Self {
        let base_url =
            base_url.unwrap_or_else(|| "https://api.anthropic.com/v1/messages".to_string());
        Self {
            client: Client::new(),
            api_key,
            model,
            temperature,
            base_url,
        }
    }
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn generate(&self, prompt: &str) -> Result<LlmResponse> {
        let req = AnthropicRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".into(),
                content: prompt.to_string(),
            }],
            temperature: self.temperature,
        };

        let res: AnthropicResponse = self
            .client
            .post(&self.base_url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&req)
            .send()
            .await
            .map_err(|e| EngineError::LlmProvider(e.to_string()))?
            .json()
            .await
            .map_err(|e| EngineError::LlmProvider(e.to_string()))?;

        let content = res
            .content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();
        Ok(LlmResponse { content })
    }
}
