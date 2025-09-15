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
use std::time::Duration;

use anyhow::Context;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Clone, ValueEnum, Debug)]
pub enum ReportFormat {
    Md,
    Json,
}

#[derive(Args, Debug)]
pub struct CheckArgs {
    /// Output format for the review report.
    #[arg(long, value_enum, default_value = "md")]
    pub format: ReportFormat,

    /// The base reference to compare against for generating a diff.
    /// Use "auto" to detect the upstream of the current branch.
    #[arg(long, default_value = "auto", alias = "base-ref")]
    pub diff: String,

    /// Run in CI mode (non-interactive).
    #[arg(long, default_value_t = false)]
    pub ci: bool,

    /// Analyze only files changed relative to the diff base. Use `--no-only-changed`
    /// to analyze all files.
    #[arg(long, default_value_t = true)]
    pub only_changed: bool,

    /// Disable progress output.
    #[arg(long, default_value_t = false)]
    pub no_progress: bool,

    /// The path to the repository to check.
    #[arg(long, default_value = ".")]
    pub path: String,

    /// The path to write the review report to.
    #[arg(short, long)]
    pub output: Option<String>,

    /// Minimum issue severity that will trigger a non-zero exit.
    /// Defaults to the `fail-on` setting in `reviewlens.toml` (`high` if unset).
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
        ReportFormat::Md => "review_report.md".to_string(),
        ReportFormat::Json => "review_report.json".to_string(),
    });

    log::info!("Running 'check' with the following arguments:");
    log::info!("  Path: {}", args.path);
    log::info!("  Output: {}", output_path);
    log::info!("  Format: {:?}", args.format);
    log::info!("  CI mode: {}", args.ci);
    log::info!("  Only changed: {}", args.only_changed);
    log::info!("  No progress: {}", args.no_progress);

    if args.ci {
        env::set_var("CI", "true");
    }
    if !args.no_progress {
        log::info!("Starting review...");
    }

    // Resolve the base reference, falling back to upstream if not provided.
    let base_ref = if args.diff != "auto" {
        args.diff.clone()
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

    // 1. Generate the diff.
    let diff_content = if args.only_changed {
        let diff_output = Command::new("git")
            .args(["-C", &args.path, "diff", &base_ref])
            .output()
            .with_context(|| "failed to execute git diff")?;
        if !diff_output.status.success() {
            anyhow::bail!("git diff command failed");
        }
        String::from_utf8(diff_output.stdout).context("diff output was not valid UTF-8")?
    } else {
        let empty_tree = Command::new("git")
            .args(["-C", &args.path, "hash-object", "-t", "tree", "/dev/null"])
            .output()
            .with_context(|| "failed to hash empty tree")?;
        if !empty_tree.status.success() {
            anyhow::bail!("git hash-object command failed");
        }
        let empty_tree_ref = String::from_utf8(empty_tree.stdout)
            .context("empty tree hash output was not valid UTF-8")?
            .trim()
            .to_string();
        let diff_output = Command::new("git")
            .args(["-C", &args.path, "diff", &empty_tree_ref])
            .output()
            .with_context(|| "failed to execute git diff")?;
        if !diff_output.status.success() {
            anyhow::bail!("git diff command failed");
        }
        String::from_utf8(diff_output.stdout).context("diff output was not valid UTF-8")?
    };

    // 2. Call the engine to run the review and capture its report.
    // Ensure file reads are relative to the provided path.
    let progress = if !args.no_progress && !args.ci {
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::with_template("{spinner} {msg}").expect("spinner template"));
        pb.enable_steady_tick(Duration::from_millis(100));
        pb.set_message("Reviewing diff...");
        Some(pb)
    } else {
        None
    };

    let report = {
        let original_dir = env::current_dir().with_context(|| "failed to get current directory")?;
        env::set_current_dir(&args.path)
            .with_context(|| format!("failed to change to directory {}", args.path))?;
        if let Some(pb) = &progress {
            pb.set_message("Running review engine...");
        }
        let result = engine
            .run(&diff_content)
            .await
            .map_err(|e| anyhow::anyhow!(e));
        env::set_current_dir(original_dir)
            .with_context(|| "failed to restore working directory")?;
        result?
    };

    if let Some(pb) = progress {
        pb.finish_and_clear();
    }

    // Print the summary and hotspots to stdout for quick visibility.
    if args.ci {
        println!("{}", report.summary);
    } else {
        println!("Summary: {}", report.summary);
        if report.hotspots.is_empty() {
            println!("No hotspots identified.");
        } else {
            println!("Top hotspots:");
            for spot in &report.hotspots {
                println!("- {}", spot);
            }
        }
    }

    // 3. Generate the report and write it to `output_path`.
    let generator: Box<dyn ReportGenerator> = match args.format {
        ReportFormat::Md => Box::new(MarkdownGenerator),
        ReportFormat::Json => Box::new(JsonGenerator),
    };
    let report_out = generator
        .generate(&report)
        .map_err(|e| anyhow::anyhow!(e))?;
    let redacted_report = redact_text(engine.config(), &report_out);
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
