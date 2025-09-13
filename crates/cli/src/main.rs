//! The command-line interface for the Intelligent Code Review Agent.

use anyhow::Context;
use clap::Parser;
use engine::ReviewEngine;
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
    #[arg(long, value_name = "PATH", default_value = "reviewer.toml")]
    config: PathBuf,

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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::init();

    let cli = Cli::parse();

    let config = engine::config::Config::load_from_path(&cli.config)
        .with_context(|| format!("failed to load config from {}", cli.config.display()))?;
    let engine = ReviewEngine::new(config);

    // Execute the subcommand
    match cli.command {
        Commands::Check(args) => commands::check::run(args, &engine).await?,
        Commands::Index(args) => commands::index::run(args, &engine).await?,
    }

    Ok(())
}
