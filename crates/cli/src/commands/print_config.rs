//! The `print-config` subcommand.

use clap::Args;
use engine::{compiled_providers, config::Config};
use std::process::Command;

use anyhow::Context;

#[derive(Args, Debug, Clone)]
pub struct PrintConfigArgs {
    /// The base reference to compare against for generating a diff.
    /// If not provided, the upstream of the current branch is used.
    #[arg(long, alias = "diff")]
    pub base_ref: Option<String>,
}

/// Executes the `print-config` subcommand.
pub fn run(args: PrintConfigArgs, config: &Config) -> anyhow::Result<()> {
    // Serialize the config to a pretty JSON string.
    let config_json = serde_json::to_string_pretty(config)?;
    log::info!("{}", config_json);

    // Resolve the base reference, falling back to upstream if not provided.
    let base_ref = if let Some(base) = args.base_ref.clone() {
        base
    } else {
        let upstream_output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])
            .output()
            .map_err(|e| anyhow::anyhow!("failed to detect upstream base: {}", e))?;
        if !upstream_output.status.success() {
            anyhow::bail!("failed to detect upstream base reference");
        }
        String::from_utf8(upstream_output.stdout)
            .context("upstream output was not valid UTF-8")?
            .trim()
            .to_string()
    };
    log::info!("Base ref: {}", base_ref);

    let providers = compiled_providers()
        .into_iter()
        .map(|p| p.as_str().to_string())
        .collect::<Vec<_>>();
    log::info!("Compiled providers: {}", providers.join(", "));
    Ok(())
}
