//! The command-line interface for the Intelligent Code Review Agent.

use clap::Parser;
use engine::{config::Config, ReviewEngine};
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
    let config = if cli.config.exists() {
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

    // The `print-config` command does not need the engine, so we handle it
    // before initializing the engine.
    if let Commands::PrintConfig(args) = &cli.command {
        return commands::print_config::run(args.clone(), &config);
    }

    let engine = ReviewEngine::new(config)?;

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
