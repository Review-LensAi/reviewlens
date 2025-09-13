//! The `check` subcommand.

use clap::Args;
use engine::config::Severity;
use engine::report::{MarkdownGenerator, ReportGenerator};
use engine::ReviewEngine;
use std::fs;
use std::process::Command;

use anyhow::Context;

#[derive(Args, Debug)]
pub struct CheckArgs {
    /// The base branch to compare against for generating a diff.
    #[arg(long, default_value = "main")]
    pub diff: String,

    /// The path to the repository to check.
    #[arg(long, default_value = ".")]
    pub path: String,

    /// The path to write the review report to.
    #[arg(short, long, default_value = "review_report.md")]
    pub output: String,

    /// Minimum issue severity that will trigger a non-zero exit.
    #[arg(long, value_enum)]
    pub fail_on: Option<Severity>,
}

/// Executes the `check` subcommand.
pub async fn run(args: CheckArgs, engine: &ReviewEngine) -> anyhow::Result<()> {
    log::info!("Running 'check' with the following arguments:");
    log::info!("  Diff base: {}", args.diff);
    log::info!("  Path: {}", args.path);
    log::info!("  Output: {}", args.output);

    // 1. Generate the diff against the specified base reference.
    let diff_output = Command::new("git")
        .args(["-C", &args.path, "diff", &args.diff])
        .output()
        .with_context(|| "failed to execute git diff")?;
    if !diff_output.status.success() {
        anyhow::bail!("git diff command failed");
    }
    let diff_content =
        String::from_utf8(diff_output.stdout).context("diff output was not valid UTF-8")?;

    // 2. Call the engine to run the review and capture its report.
    let report = engine
        .run(&diff_content)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    // 3. Generate the markdown report and write it to `args.output`.
    let generator = MarkdownGenerator;
    let report_md = generator
        .generate(&report)
        .map_err(|e| anyhow::anyhow!(e))?;
    fs::write(&args.output, &report_md)?;
    log::info!("\nReview complete. Report written to {}.", args.output);

    // 4. Exit non-zero based on severity threshold.
    if let Some(threshold) = args.fail_on {
        let max_severity = report
            .issues
            .iter()
            .map(|issue| issue.severity.clone())
            .max();
        if let Some(max) = max_severity {
            if max >= threshold {
                anyhow::bail!(
                    "Issues of severity {:?} or higher were found in the diff",
                    threshold
                );
            }
        }
    }

    Ok(())
}
