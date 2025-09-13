//! The `version` subcommand.

use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct VersionArgs {}

/// Prints the binary version.
pub fn run(_args: VersionArgs) -> anyhow::Result<()> {
    println!("{}", env!("CARGO_PKG_VERSION"));
    Ok(())
}
