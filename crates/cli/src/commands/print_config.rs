//! The `print-config` subcommand.

use clap::Args;
use engine::config::Config;

#[derive(Args, Debug, Clone)]
pub struct PrintConfigArgs {}

/// Executes the `print-config` subcommand.
pub fn run(_args: PrintConfigArgs, config: &Config) -> anyhow::Result<()> {
    // Serialize the config to a pretty JSON string.
    let config_json = serde_json::to_string_pretty(config)?;
    println!("{}", config_json);
    Ok(())
}
