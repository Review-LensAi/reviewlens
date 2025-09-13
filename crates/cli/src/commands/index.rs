//! The `index` subcommand.

use clap::Args;
use engine::rag::index_repository;
use engine::ReviewEngine;

#[derive(Args, Debug)]
pub struct IndexArgs {
    /// The path to the repository to index.
    #[arg(long, default_value = ".")]
    pub path: String,

    /// If set, forces a full re-indexing, ignoring any existing cache.
    #[arg(long)]
    pub force: bool,
}

/// Executes the `index` subcommand.
pub async fn run(args: IndexArgs, _engine: &ReviewEngine) -> anyhow::Result<()> {
    let _store = index_repository(&args.path, args.force).await?;
    println!("Indexing complete.");
    Ok(())
}
