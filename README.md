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

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Git

### Installation & Setup

1.  **Clone the repository:**
    ```bash
    git clone <repository_url>
    cd intelligent-code-reviewer
    ```

2.  **Configuration:**
    Copy the example configuration file and customize it.
    ```bash
    cp reviewer.toml.example reviewer.toml
    ```
    Edit `reviewer.toml` to select your desired LLM provider, model, paths, and rule settings.

3.  **Build the CLI:**
    ```bash
    cargo build --release
    ```
    The binary will be available at `target/release/reviewer-cli`.

### Usage

The primary command is `reviewer-cli check`. It analyzes the difference between your current branch and a base branch (`main` by default).

```bash
# Run a review against the 'main' branch
./target/release/reviewer-cli check --diff main

# The review will be saved to `review_report.md`
cat review_report.md
```

## Architecture

The project is structured as a Cargo workspace:

-   `crates/engine`: The core library containing all analysis, RAG, scanning, and reporting logic. It is completely independent of the CLI.
-   `crates/cli`: A thin wrapper around the `engine` that provides a command-line interface.
-   `reviewer.toml`: The configuration file for defining project rules, LLM providers, and other settings.

## CI/CD Integration

You can run the agent in your CI pipeline to automatically review merge requests. The CLI is designed to exit with a non-zero status code if issues are found, allowing you to gate PRs.

See the `docs/ci/` directory for example configurations for GitHub Actions and GitLab CI.

---

*This project was bootstrapped by an AI agent.*
