//! The command-line interface for the Intelligent Code Review Agent.

use clap::Parser;
use engine::{
    config::{Config, IndexConfig, Provider},
    error::EngineError,
    ReviewEngine,
};
use env_logger::Target;
use log::LevelFilter;
use std::io::Write;
use std::path::PathBuf;

mod commands;

/// A context-aware, security-first code review agent that runs locally or in CI.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Sets the verbosity level.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Path to configuration file.
    #[arg(long, value_name = "PATH", default_value = "reviewlens.toml")]
    config: PathBuf,

    /// Override the LLM provider.
    #[arg(long, value_enum, env = "REVIEWLENS_LLM_PROVIDER")]
    llm_provider: Option<Provider>,

    /// Override the LLM model.
    #[arg(long, env = "REVIEWLENS_LLM_MODEL")]
    llm_model: Option<String>,

    /// Override the LLM API key.
    #[arg(long, env = "REVIEWLENS_LLM_API_KEY")]
    llm_api_key: Option<String>,

    /// Override the LLM base URL.
    #[arg(long, env = "REVIEWLENS_LLM_BASE_URL")]
    llm_base_url: Option<String>,

    /// Override the path to the RAG index.
    #[arg(long, env = "REVIEWLENS_INDEX_PATH")]
    index_path: Option<String>,

    /// Override token budget per run.
    #[arg(long, env = "REVIEWLENS_BUDGET_TOKENS_MAX_PER_RUN")]
    budget_tokens_max_per_run: Option<u32>,

    /// Override generation temperature.
    #[arg(long, env = "REVIEWLENS_GENERATION_TEMPERATURE")]
    generation_temperature: Option<f32>,

    /// Override allowed paths (comma separated).
    #[arg(long, value_delimiter = ',', env = "REVIEWLENS_PATHS_ALLOW")]
    paths_allow: Vec<String>,

    /// Override denied paths (comma separated).
    #[arg(long, value_delimiter = ',', env = "REVIEWLENS_PATHS_DENY")]
    paths_deny: Vec<String>,

    /// Enable or disable redaction.
    #[arg(long, env = "REVIEWLENS_PRIVACY_REDACTION_ENABLED")]
    privacy_redaction_enabled: Option<bool>,

    /// Override redaction patterns (comma separated).
    #[arg(
        long,
        value_delimiter = ',',
        env = "REVIEWLENS_PRIVACY_REDACTION_PATTERNS"
    )]
    privacy_redaction_patterns: Vec<String>,

    #[command(subcommand)]
    command: Commands,
}

/// The subcommands for the CLI.
#[derive(Parser, Debug)]
enum Commands {
    /// Checks a diff for issues and generates a review report.
    Check(commands::check::CheckArgs),
    /// Manages the RAG index for a repository.
    Index(commands::index::IndexArgs),
    /// Prints the effective configuration, compiled providers, and resolved base reference.
    PrintConfig(commands::print_config::PrintConfigArgs),
    /// Prints the CLI version.
    Version(commands::version::VersionArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let mut builder =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"));
    builder.filter_level(match cli.verbose {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    });
    if matches!(cli.command, Commands::PrintConfig(_)) && cli.verbose == 0 {
        builder.filter_level(LevelFilter::Info);
    }
    builder.target(Target::Stdout);
    builder.format(|f, record| writeln!(f, "{}", record.args()));
    builder.init();

    if let Commands::Version(args) = &cli.command {
        return commands::version::run(args.clone());
    }

    // Load configuration from the path specified in the CLI arguments.
    // If the file doesn't exist, use the default configuration.
    let mut config = if cli.config.exists() {
        if !matches!(cli.command, Commands::PrintConfig(_)) {
            log::info!("Loading configuration from: {:?}", cli.config);
        }
        Config::load_from_path(&cli.config)?
    } else {
        if !matches!(cli.command, Commands::PrintConfig(_)) {
            log::info!(
                "Configuration file {:?} not found. Using default configuration.",
                cli.config
            );
        }
        Config::default()
    };

    // Apply environment variable and CLI overrides.
    if let Some(p) = cli.llm_provider {
        config.llm.provider = p;
    }
    if let Some(model) = cli.llm_model {
        config.llm.model = Some(model);
    }
    if let Some(key) = cli.llm_api_key {
        config.llm.api_key = Some(key);
    }
    if let Some(url) = cli.llm_base_url {
        config.llm.base_url = Some(url);
    }
    if let Some(path) = cli.index_path {
        config.index = Some(IndexConfig { path });
    }
    if let Some(max) = cli.budget_tokens_max_per_run {
        config.budget.tokens.max_per_run = Some(max);
    }
    if let Some(temp) = cli.generation_temperature {
        config.generation.temperature = Some(temp);
    }
    if !cli.paths_allow.is_empty() {
        config.paths.allow = cli.paths_allow.clone();
    }
    if !cli.paths_deny.is_empty() {
        config.paths.deny = cli.paths_deny.clone();
    }
    if let Some(enabled) = cli.privacy_redaction_enabled {
        config.privacy.redaction.enabled = enabled;
    }
    if !cli.privacy_redaction_patterns.is_empty() {
        config.privacy.redaction.patterns = cli.privacy_redaction_patterns.clone();
    }

    // The `print-config` command does not need the engine, so we handle it
    // before initializing the engine.
    if let Commands::PrintConfig(args) = &cli.command {
        return commands::print_config::run(args.clone(), &config);
    }

    let engine = match ReviewEngine::new(config) {
        Ok(engine) => engine,
        Err(e) => {
            log::error!("{}", e);
            match e {
                EngineError::Config(_) => std::process::exit(2),
                _ => std::process::exit(3),
            }
        }
    };

    // Execute the subcommand
    match cli.command {
        Commands::Check(args) => {
            let code = commands::check::run(args, &engine).await;
            std::process::exit(code);
        }
        Commands::Index(args) => commands::index::run(args, &engine).await?,
        Commands::PrintConfig(_) => {
            // This case is handled above, but the compiler needs it to be exhaustive.
            unreachable!()
        }
        Commands::Version(_) => {
            // This case is handled above, but the compiler needs it to be exhaustive.
            unreachable!()
        }
    }

    Ok(())
}
