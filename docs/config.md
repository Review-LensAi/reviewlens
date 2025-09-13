# Configuration

`reviewer.toml` controls how the agent behaves. Values are merged in this order of precedence:
1. CLI flags
2. Environment variables (prefixed with `REVIEWER_`)
3. Settings in `reviewer.toml`

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
path = ".reviewer/index/index.json"
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

## Using in CI
Supply sensitive values such as API keys via environment variables in your CI system. Example GitHub Actions and GitLab CI files live in [`docs/ci/`](ci/).
