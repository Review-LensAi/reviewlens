//! The command-line interface for the Intelligent Code Review Agent.

use clap::Parser;
use engine::ReviewEngine;

mod commands;

/// A context-aware, security-first code review agent that runs locally or in CI.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Sets the verbosity level.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

/// The subcommands for the CLI.
#[derive(Parser, Debug)]
enum Commands {
    /// Checks a diff for issues and generates a review report.
    Check(commands::check::CheckArgs),
    /// Manages the RAG index for a repository.
    Index(commands::index::IndexArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::init();

    let cli = Cli::parse();

    // Placeholder: Load config and initialize the engine
    // In a real app, you'd load this from a `reviewer.toml` file.
    let config = engine::config::Config {
        llm: engine::config::LlmConfig {
            provider: "local".to_string(),
            model: "dummy".to_string(),
            temperature: 0.1,
        },
        project: engine::config::ProjectConfig {
            include: vec!["**/*".to_string()],
            exclude: vec!["target/*".to_string(), ".git/*".to_string()],
        },
        rules: engine::config::RulesConfig {
            owasp_top_5: true,
            secrets: true,
        },
    };
    let engine = ReviewEngine::new(config);

    // Execute the subcommand
    match cli.command {
        Commands::Check(args) => commands::check::run(args, &engine).await?,
        Commands::Index(args) => commands::index::run(args, &engine).await?,
    }

    Ok(())
}
