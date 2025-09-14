# Configuration

`reviewlens.toml` controls how the agent behaves. Values are merged in this order of precedence:
1. CLI flags
2. Environment variables (prefixed with `REVIEWLENS_`)
3. Settings in `reviewlens.toml`

## Paths
Define which files are scanned:
```toml
[paths]
allow = ["src/**/*.rs", "crates/**/*.rs"]
deny  = ["target/*", "**/testdata/*"]
```
Only files in `paths.allow` are indexed, helping enforce repository boundaries.

## Index

Override the location of the pre-built vector index:

```toml
[index]
path = ".reviewlens/index/index.json"
```

The older top-level `index-path` setting is deprecated.

## LLM Provider
```toml
[llm]
provider = "null"
model = "gpt-4-turbo"
# api_key = "YOUR_API_KEY"
```
Set `provider` and `api_key` to use a remote model. The default `provider = "null"` keeps all analysis local.

## Privacy
```toml
[privacy.redaction]
enabled = true
patterns = []
```
Secret redaction is enabled by default, and additional patterns can be supplied. Combine this with path allowlists to ensure code privacy.

## Budget and Generation
Optional sections let you cap token usage or adjust generation parameters:
```toml
[budget.tokens]
# max-per-run = 100000

[generation]
temperature = 0.0
```

## Diagrams
When three or more changed files reference one another, the engine populates `mermaid_diagram` in the `ReviewReport` with a simple Mermaid sequence diagram. The Markdown report renders this automatically; no additional configuration is required.

## Hotspot Weights
Rank hotspots by combining scanner findings and code churn:
```toml
[report.hotspot_weights]
severity = 3
churn = 1
```
Higher `severity` favors files with more findings, while `churn` boosts files with more changed lines.

## Using in CI
Supply sensitive values such as API keys via environment variables in your CI system. Example GitHub Actions and GitLab CI files live in [`docs/ci/`](ci/).
