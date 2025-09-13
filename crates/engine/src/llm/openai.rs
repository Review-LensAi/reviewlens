use super::{LlmProvider, LlmResponse};
use crate::error::{EngineError, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    model: String,
    temperature: f32,
    base_url: String,
}

impl OpenAiProvider {
    pub fn new(api_key: String, model: String, temperature: f32, base_url: Option<String>) -> Self {
        let base_url =
            base_url.unwrap_or_else(|| "https://api.openai.com/v1/chat/completions".to_string());
        Self {
            client: Client::new(),
            api_key,
            model,
            temperature,
            base_url,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Deserialize)]
struct ChatCompletionChoice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
    #[serde(default)]
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn generate(&self, prompt: &str) -> Result<LlmResponse> {
        let req = ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![ChatMessage {
                role: "user".into(),
                content: prompt.to_string(),
            }],
            temperature: self.temperature,
        };

        let res: ChatCompletionResponse = self
            .client
            .post(&self.base_url)
            .bearer_auth(&self.api_key)
            .json(&req)
            .send()
            .await
            .map_err(|e| EngineError::LlmProvider(e.to_string()))?
            .json()
            .await
            .map_err(|e| EngineError::LlmProvider(e.to_string()))?;

        let content = res
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();
        let tokens = res.usage.map(|u| u.total_tokens).unwrap_or(0);
        Ok(LlmResponse {
            content,
            token_usage: tokens,
        })
    }
}
