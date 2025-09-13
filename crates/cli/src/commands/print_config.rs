//! The `print-config` subcommand.

use clap::Args;
use engine::{compiled_providers, config::Config};

#[derive(Args, Debug, Clone)]
pub struct PrintConfigArgs {}

/// Executes the `print-config` subcommand.
pub fn run(_args: PrintConfigArgs, config: &Config) -> anyhow::Result<()> {
    // Serialize the config to a pretty JSON string.
    let config_json = serde_json::to_string_pretty(config)?;
    log::info!("{}", config_json);
    let providers = compiled_providers()
        .into_iter()
        .map(|p| p.as_str().to_string())
        .collect::<Vec<_>>();
    log::info!("Compiled providers: {}", providers.join(", "));
    Ok(())
}
