//! The `check` subcommand.

use clap::{Args, ValueEnum};
use engine::config::Severity;
use engine::error::EngineError;
use engine::redact_text;
use engine::report::{JsonGenerator, MarkdownGenerator, ReportGenerator};
use engine::ReviewEngine;
use std::env;
use std::fs;
use std::process::Command;

use anyhow::Context;

#[derive(ValueEnum, Clone, Debug)]
pub enum ReportFormat {
    Markdown,
    Json,
}

#[derive(Args, Debug)]
pub struct CheckArgs {
    /// The base reference to compare against for generating a diff.
    /// If not provided, the upstream of the current branch is used.
    #[arg(long, alias = "diff")]
    pub base_ref: Option<String>,

    /// The path to the repository to check.
    #[arg(long, default_value = ".")]
    pub path: String,

    /// The path to write the review report to.
    #[arg(short, long)]
    pub output: Option<String>,

    /// The format of the review report.
    #[arg(long, value_enum, default_value_t = ReportFormat::Markdown)]
    pub format: ReportFormat,

    /// Minimum issue severity that will trigger a non-zero exit.
    /// Defaults to the `fail-on` setting in `reviewlens.toml` (`low` if unset).
    #[arg(long, value_enum)]
    pub fail_on: Option<Severity>,
}

/// Executes the `check` subcommand.
/// Returns the appropriate exit code.
pub async fn run(args: CheckArgs, engine: &ReviewEngine) -> i32 {
    match execute(args, engine).await {
        Ok(issues_found) => {
            if issues_found {
                1
            } else {
                0
            }
        }
        Err(e) => {
            if let Some(engine_error) = e.downcast_ref::<EngineError>() {
                match engine_error {
                    EngineError::Config(_) => {
                        log::error!("{}", e);
                        2
                    }
                    _ => {
                        log::error!("{}", e);
                        3
                    }
                }
            } else {
                log::error!("{}", e);
                3
            }
        }
    }
}

async fn execute(args: CheckArgs, engine: &ReviewEngine) -> anyhow::Result<bool> {
    let output_path = args.output.clone().unwrap_or_else(|| match args.format {
        ReportFormat::Markdown => "review_report.md".to_string(),
        ReportFormat::Json => "review_report.json".to_string(),
    });

    log::info!("Running 'check' with the following arguments:");
    log::info!("  Path: {}", args.path);
    log::info!("  Format: {:?}", args.format);
    log::info!("  Output: {}", output_path);

    // Resolve the base reference, falling back to upstream if not provided.
    let base_ref = if let Some(base) = args.base_ref.clone() {
        base
    } else {
        let upstream_output = Command::new("git")
            .args([
                "-C",
                &args.path,
                "rev-parse",
                "--abbrev-ref",
                "--symbolic-full-name",
                "@{u}",
            ])
            .output()
            .map_err(|e| EngineError::Config(format!("failed to detect upstream base: {}", e)))?;
        if !upstream_output.status.success() {
            return Err(
                EngineError::Config("failed to detect upstream base reference".into()).into(),
            );
        }
        String::from_utf8(upstream_output.stdout)
            .context("upstream output was not valid UTF-8")?
            .trim()
            .to_string()
    };
    log::info!("  Base ref: {}", base_ref);

    // 1. Generate the diff against the base reference.
    let diff_output = Command::new("git")
        .args(["-C", &args.path, "diff", &base_ref])
        .output()
        .with_context(|| "failed to execute git diff")?;
    if !diff_output.status.success() {
        anyhow::bail!("git diff command failed");
    }
    let diff_content =
        String::from_utf8(diff_output.stdout).context("diff output was not valid UTF-8")?;

    // 2. Call the engine to run the review and capture its report.
    // Ensure file reads are relative to the provided path.
    let report = {
        let original_dir = env::current_dir().with_context(|| "failed to get current directory")?;
        env::set_current_dir(&args.path)
            .with_context(|| format!("failed to change to directory {}", args.path))?;
        let result = engine
            .run(&diff_content)
            .await
            .map_err(|e| anyhow::anyhow!(e));
        env::set_current_dir(original_dir)
            .with_context(|| "failed to restore working directory")?;
        result?
    };

    // Print the summary and hotspots to stdout for quick visibility.
    println!("Summary: {}", report.summary);
    if report.hotspots.is_empty() {
        println!("No hotspots identified.");
    } else {
        println!("Top hotspots:");
        for spot in &report.hotspots {
            println!("- {}", spot);
        }
    }

    // 3. Generate the report and write it to the selected output path.
    let report_str = match args.format {
        ReportFormat::Markdown => {
            let generator = MarkdownGenerator;
            generator
                .generate(&report)
                .map_err(|e| anyhow::anyhow!(e))?
        }
        ReportFormat::Json => {
            let generator = JsonGenerator;
            generator
                .generate(&report)
                .map_err(|e| anyhow::anyhow!(e))?
        }
    };
    let redacted_report = redact_text(engine.config(), &report_str);
    fs::write(&output_path, &redacted_report)?;
    log::info!("\nReview complete. Report written to {}.", output_path);

    // 4. Determine if issues exceed the severity threshold.
    let threshold = args
        .fail_on
        .unwrap_or_else(|| engine.config().fail_on.clone());
    let issues_found = report
        .issues
        .iter()
        .map(|issue| issue.severity.clone())
        .max()
        .map_or(false, |max| max >= threshold);

    Ok(issues_found)
}
