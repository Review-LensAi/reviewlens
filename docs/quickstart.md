# Quickstart

This guide helps you install and run the `reviewlens` locally or in CI.

## Installation
- Use the install script:
  ```bash
  curl -fsSL https://raw.githubusercontent.com/Review-LensAi/reviewlens/main/install.sh | sh
  ```
- Download a release binary and place it in your `PATH`.
- Or build from source with `cargo install`.

## Configuration
1. Copy the example file:
   ```bash
   cp reviewlens.toml.example reviewlens.toml
   ```
2. Set your LLM provider and API key. Configuration values can also be supplied via environment variables or CLI flags.

## Inspect Effective Configuration
After configuring, you can inspect the merged settings, compiled providers, and the detected base reference:
```bash
reviewlens print-config
```
Look for the `Base ref:` line in the output to see which upstream branch will be used for diffs.

## Running a Review
Build the RAG cache for your project before running the agent:
```bash
reviewlens index --path .
```
This writes `.reviewlens/index/index.json`. Use `--force` to refresh the cache after major file changes.

Then run the agent from the root of your project:
```bash
reviewlens check --base-ref main
```
The CLI prints a short summary and the top hotspots to stdout, while the full report is written to `review_report.md`.

When three or more files reference one another, the report also includes a Mermaid sequence diagram visualizing the flow between them.

## CI Setup
The CLI can gate pull requests by exiting non‑zero when issues are found. See the sample configurations in [`docs/ci/`](ci/) for GitHub Actions and GitLab CI examples.

## Privacy Defaults
The tool is designed with privacy in mind:
- Only files listed in `paths.allow` are analyzed.
- Secret redaction is enabled by default (`privacy.redaction.enabled = true`).
- The example configuration uses `provider = "null"`, so no code is sent to an external LLM unless you explicitly configure one.
