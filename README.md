# Intelligent Code Review Agent

A context-aware, security-first code review agent that runs locally or in CI to summarize changes, flag real issues, and suggest precise fixes—without vendor lock-in to any single LLM.

## Vision

Our goal is to build an agent that slashes review time while improving code quality and security. It understands codebase context to provide relevant suggestions, catches high-impact issues early, and delivers actionable, concise feedback.

## Core Principles

- **CLI First:** The primary interface is a powerful and scriptable command-line tool.
- **Engine as a Library:** The core logic is a separate `engine` crate, allowing it to be used by the CLI, CI bots, IDE plugins, and other applications.
- **Provider-Agnostic:** A clean trait-based abstraction for LLMs ensures we are not locked into any single provider. Supports OpenAI, Anthropic, DeepSeek, and local/self-hosted models.
- **Config over Code:** Behavior is controlled through a simple `reviewer.toml` file.
- **Security & Privacy by Default:** Features path allowlists, secret redaction, and an offline "local-only" mode to ensure code privacy.

## Installation

We offer several methods to install the `reviewer-cli`. Choose the one that best fits your workflow.

### Install Script (Recommended for Linux & macOS)

You can install the latest version using our installer script. It will automatically detect your OS and install the correct pre-compiled binary.

```bash
curl -fsSL https://raw.githubusercontent.com/some-org/intelligent-code-reviewer/main/install.sh | sh
```

The script will place the `reviewer-cli` binary in `/usr/local/bin` and may prompt for `sudo` access.

### GitHub Releases (Linux, macOS, Windows)

You can download pre-compiled binaries directly from the [GitHub Releases page](https://github.com/some-org/intelligent-code-reviewer/releases).

Download the appropriate archive for your operating system, extract it, and place the `reviewer-cli` (or `reviewer-cli.exe`) binary in a directory included in your system's `PATH`.

Each release also provides a `.sha256` checksum and a `.sig` signature file generated with [cosign](https://github.com/sigstore/cosign). After downloading an archive, you can verify its integrity and authenticity:

```bash
# Verify the checksum
sha256sum -c reviewer-cli-<TARGET>.tar.gz.sha256

# Verify the signature (requires cosign)
cosign verify-blob --signature reviewer-cli-<TARGET>.tar.gz.sig reviewer-cli-<TARGET>.tar.gz
```

### With `cargo` (Requires Rust)

If you have the Rust toolchain installed, you can build and install `reviewer-cli` from crates.io.

```bash
cargo install reviewer-cli
```
*(Note: The crate is not yet published. This will be available in a future release.)*

### With Docker

For a containerized environment, you can use Docker.

1.  **Build the image:**
    ```bash
    docker build -t reviewer-cli .
    ```

2.  **Run the container:**
    You'll need to mount your project directory and pass your configuration.
    ```bash
    docker run --rm -v "$(pwd):/work" reviewer-cli check --base-ref main
    ```

### From Source

If you prefer to build from source, you can clone the repository and build the CLI manually.

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/some-org/intelligent-code-reviewer.git
    cd intelligent-code-reviewer
    ```

2.  **Build the CLI:**
    ```bash
    cargo build --release
    ```
    The binary will be available at `target/release/reviewer-cli`. You can add this to your `PATH` or use it directly.


## Getting Started

After installing `reviewer-cli`, follow these steps to get started.

### 1. Configuration

The agent is controlled by a `reviewer.toml` file. Copy the example file to the root of your project:

```bash
cp reviewer.toml.example reviewer.toml
```

Next, edit `reviewer.toml` to configure your desired LLM provider, model, project paths, and review rules. At a minimum, you must set your LLM provider and API key.

Configuration values are merged from multiple sources. The precedence is:

1. CLI flags
2. Environment variables (prefixed with `REVIEWER_`)
3. Values from `reviewer.toml`

For example, `--llm-provider anthropic` overrides `REVIEWER_LLM_PROVIDER`, which in turn overrides the `llm.provider` value in the configuration file.

### 2. Usage

The primary command is `reviewer-cli check`. It analyzes the difference between your current branch and a base branch (e.g., `main`).

Run a review from the root of your project:
```bash
# Run a review against the 'main' branch
reviewer-cli check --base-ref main
```

The review report will be saved to `review_report.md` by default. You can view it with:
```bash
cat review_report.md
```

## CI/CD Integration

You can run the agent in your CI pipeline to automatically review merge requests. The CLI is designed to exit with a non-zero status code if issues are found, allowing you to gate PRs.

See the `docs/ci/` directory for example configurations for GitHub Actions and GitLab CI.

## Documentation
- [Quickstart](docs/quickstart.md) – install and run the agent, including CI setup and privacy defaults.
- [Configuration](docs/config.md) – list of options and default privacy settings.
- [Troubleshooting](docs/troubleshooting.md) – common errors and fixes.

## Architecture

The project is structured as a Cargo workspace:

-   `crates/engine`: The core library containing all analysis, RAG, scanning, and reporting logic.
-   `crates/cli`: A thin wrapper around the `engine` that provides a command-line interface.
-   `reviewer.toml`: The configuration file for defining project rules, LLM providers, and other settings.

## Supported Diff Formats

The engine uses the [`patch`](https://crates.io/crates/patch) crate to parse diffs in the unified format. It understands
standard text diffs, file renames, binary file changes, and multiple hunks within a single file.

## Contributing
See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Code of Conduct
Please see [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## Security
For reporting vulnerabilities, see [SECURITY.md](SECURITY.md).

## License
Licensed under the [Apache-2.0](LICENSE) license.

---

*This project was bootstrapped by an AI agent.*
