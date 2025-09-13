//! Configuration structures for the code review agent.
//!
//! This module defines the structs that can be deserialized from the
//! `reviewer.toml` configuration file.

use crate::error::{EngineError, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub llm: LlmConfig,
    pub project: ProjectConfig,
    #[serde(default)]
    pub rules: RulesConfig,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum Provider {
    Openai,
    Anthropic,
    Deepseek,
    Local,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct LlmConfig {
    pub provider: Provider,
    pub model: String,
    #[serde(default)]
    pub temperature: f32,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ProjectConfig {
    /// Paths to include in the analysis. Globs are supported.
    #[serde(default = "default_include")]
    pub include: Vec<String>,
    /// Paths to exclude from the analysis. Globs are supported.
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct RulesConfig {
    // Configuration for different scanners can go here.
    #[serde(default)]
    pub owasp_top_5: bool,
    #[serde(default)]
    pub secrets: bool,
}

fn default_include() -> Vec<String> {
    vec!["**/*".to_string()]
}

impl Config {
    /// Loads configuration from a TOML file.
    pub fn load_from_path(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content).map_err(|e| EngineError::Config(e.to_string()))
    }
}
