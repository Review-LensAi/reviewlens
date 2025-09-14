//! Configuration structures for the code review agent.
//!
//! This module defines the structs that can be deserialized from the
//! `reviewlens.toml` configuration file.

use crate::error::{EngineError, Result};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Default path for the RAG index file.
pub const DEFAULT_INDEX_PATH: &str = ".reviewlens/index/index.json";

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct IndexConfig {
    pub path: String,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            path: DEFAULT_INDEX_PATH.to_string(),
        }
    }
}

// As per PRD section 9
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub budget: BudgetConfig,
    #[serde(default)]
    pub generation: GenerationConfig,
    #[serde(default)]
    pub privacy: PrivacyConfig,
    #[serde(default)]
    pub paths: PathsConfig,
    /// Configuration for the pre-built vector index used for RAG.
    #[serde(default)]
    pub report: ReportConfig,
    /// Optional path to a pre-built vector index used for RAG.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub index: Option<IndexConfig>,
    #[deprecated(note = "use [index] table instead")]
    #[serde(skip_serializing, default)]
    pub index_path: Option<String>,
    #[serde(default)]
    pub rules: RulesConfig,
    #[serde(default = "default_fail_on")]
    pub fail_on: Severity,
}

// As per PRD: `null | openai | anthropic | deepseek`
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Provider {
    #[serde(rename = "null")]
    Null,
    Openai,
    Anthropic,
    Deepseek,
}

impl Provider {
    /// Returns the kebab-case name of the provider.
    pub fn as_str(&self) -> &'static str {
        match self {
            Provider::Null => "null",
            Provider::Openai => "openai",
            Provider::Anthropic => "anthropic",
            Provider::Deepseek => "deepseek",
        }
    }
}

// Default provider is "null"
impl Default for Provider {
    fn default() -> Self {
        Provider::Null
    }
}

// As per PRD: `[llm]` section
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct LlmConfig {
    #[serde(default)]
    pub provider: Provider,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>, // Model is optional, especially for null provider
    #[serde(skip_serializing)]
    pub api_key: Option<String>, // Keep for actual implementations, but don't print it
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>, // Keep for actual implementations
}

// Default LLM config
impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: Provider::Null,
            model: None,
            api_key: None,
            base_url: None,
        }
    }
}

// As per PRD: `[budget.tokens]` section
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct TokenBudgetConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_per_run: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct BudgetConfig {
    #[serde(default)]
    pub tokens: TokenBudgetConfig,
}

// As per PRD: `[generation]` section
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

// As per PRD: `[privacy.redaction]` section
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct RedactionConfig {
    pub enabled: bool,
    #[serde(default)]
    pub patterns: Vec<String>,
}

impl Default for RedactionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            patterns: vec![
                "(?i)api[_-]?key".to_string(),
                "aws_secret_access_key".to_string(),
                "token".to_string(),
            ],
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct PrivacyConfig {
    #[serde(default)]
    pub redaction: RedactionConfig,
}

// As per PRD: `[paths]` section. Renaming ProjectConfig.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct PathsConfig {
    /// Paths to include in the analysis. Globs are supported.
    #[serde(default = "default_include")]
    pub allow: Vec<String>,
    /// Paths to exclude from the analysis. Globs are supported.
    #[serde(default)]
    pub deny: Vec<String>,
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self {
            allow: default_include(),
            deny: vec![],
        }
    }
}

fn default_include() -> Vec<String> {
    vec!["**/*".to_string()]
}

// As per PRD: `[report.hotspot_weights]` section
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct HotspotWeights {
    #[serde(default = "default_severity_weight")]
    pub severity: u32,
    #[serde(default = "default_churn_weight")]
    pub churn: u32,
}

impl Default for HotspotWeights {
    fn default() -> Self {
        Self {
            severity: default_severity_weight(),
            churn: default_churn_weight(),
        }
    }
}

fn default_severity_weight() -> u32 {
    3
}

fn default_churn_weight() -> u32 {
    1
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct ReportConfig {
    #[serde(default)]
    pub hotspot_weights: HotspotWeights,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            hotspot_weights: HotspotWeights::default(),
        }
    }
}

// As per PRD: `[rules]` section with severity
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

impl Severity {
    fn as_u8(&self) -> u8 {
        match self {
            Severity::Critical => 4,
            Severity::High => 3,
            Severity::Medium => 2,
            Severity::Low => 1,
        }
    }
}

impl PartialOrd for Severity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_u8().partial_cmp(&other.as_u8())
    }
}

impl Ord for Severity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_u8().cmp(&other.as_u8())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct RuleConfig {
    pub enabled: bool,
    pub severity: Severity,
}

// Sensible defaults for a rule. Let's say enabled by default with medium severity.
impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            severity: Severity::Medium,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct RulesConfig {
    #[serde(default = "default_secrets_rule")]
    pub secrets: RuleConfig,
    #[serde(default = "default_sql_injection_go_rule")]
    pub sql_injection_go: RuleConfig,
    #[serde(default = "default_http_timeouts_go_rule")]
    pub http_timeouts_go: RuleConfig,
}

fn default_secrets_rule() -> RuleConfig {
    RuleConfig {
        enabled: true,
        severity: Severity::High,
    }
}

fn default_sql_injection_go_rule() -> RuleConfig {
    RuleConfig {
        enabled: true,
        severity: Severity::Critical,
    }
}

fn default_http_timeouts_go_rule() -> RuleConfig {
    RuleConfig {
        enabled: true,
        severity: Severity::Medium,
    }
}

impl Default for RulesConfig {
    fn default() -> Self {
        Self {
            secrets: default_secrets_rule(),
            sql_injection_go: default_sql_injection_go_rule(),
            http_timeouts_go: default_http_timeouts_go_rule(),
        }
    }
}

impl Config {
    /// Loads configuration from a TOML file.
    pub fn load_from_path(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content).map_err(|e| EngineError::Config(e.to_string()))
    }

    /// Returns the configured index path, respecting the deprecated field.
    pub fn index_path(&self) -> Option<&str> {
        if let Some(index) = &self.index {
            Some(index.path.as_str())
        } else {
            #[allow(deprecated)]
            {
                self.index_path.as_deref()
            }
        }
    }
}

// Need a Default implementation for Config as well, so we can create one if the file is missing.
impl Default for Config {
    fn default() -> Self {
        Self {
            llm: LlmConfig::default(),
            budget: BudgetConfig::default(),
            generation: GenerationConfig::default(),
            privacy: PrivacyConfig::default(),
            paths: PathsConfig::default(),
            index: Some(IndexConfig::default()),
            #[allow(deprecated)]
            index_path: None,
            report: ReportConfig::default(),
            rules: RulesConfig::default(),
            fail_on: default_fail_on(),
        }
    }
}

fn default_fail_on() -> Severity {
    Severity::Low
}
