//! The command-line interface for the Intelligent Code Review Agent.

use clap::Parser;
use engine::{config::Config, ReviewEngine};
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
    /// Prints the effective configuration.
    PrintConfig(commands::print_config::PrintConfigArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::init();

    let cli = Cli::parse();

    // Load configuration from the path specified in the CLI arguments.
    // If the file doesn't exist, use the default configuration.
    let config = if cli.config.exists() {
        log::info!("Loading configuration from: {:?}", cli.config);
        Config::load_from_path(&cli.config)?
    } else {
        log::info!(
            "Configuration file {:?} not found. Using default configuration.",
            cli.config
        );
        Config::default()
    };

    // The `print-config` command does not need the engine, so we handle it
    // before initializing the engine.
    if let Commands::PrintConfig(args) = &cli.command {
        return commands::print_config::run(args.clone(), &config);
    }

    let engine = ReviewEngine::new(config)?;

    // Execute the subcommand
    match cli.command {
        Commands::Check(args) => commands::check::run(args, &engine).await?,
        Commands::Index(args) => commands::index::run(args, &engine).await?,
        Commands::PrintConfig(_) => {
            // This case is handled above, but the compiler needs it to be exhaustive.
            unreachable!()
        }
    }

    Ok(())
}
