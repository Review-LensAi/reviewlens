//! The `index` subcommand.

use clap::Args;
use engine::rag::index_repository;
use engine::ReviewEngine;
use std::fs;
use std::path::Path;

#[derive(Args, Debug)]
pub struct IndexArgs {
    /// The path to the repository to index.
    #[arg(long, default_value = ".")]
    pub path: String,

    /// If set, forces a full re-indexing, ignoring any existing cache.
    #[arg(long)]
    pub force: bool,

    /// The path to write the generated index to.
    #[arg(long, default_value = "index.json")]
    pub output: String,
}

/// Executes the `index` subcommand.
pub async fn run(args: IndexArgs, _engine: &ReviewEngine) -> anyhow::Result<()> {
    log::info!("Running 'index' with the following arguments:");
    log::info!("  Path: {}", args.path);
    log::info!("  Force: {}", args.force);
    log::info!("  Output: {}", args.output);

    // Build the index using the engine's repository indexer.
    let store = index_repository(&args.path, args.force)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    // Persist the index to the requested location.
    log::info!(
        "Index built with {} documents. Persisting to {}",
        store.len(),
        args.output
    );
    if let Some(parent) = Path::new(&args.output).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let file = fs::File::create(&args.output)?;
    serde_json::to_writer_pretty(file, &store)?;
    log::info!("Index written to {}", args.output);

    Ok(())
}
