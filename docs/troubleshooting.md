# Troubleshooting

Common issues and their fixes.

## Missing API Key
The CLI will report an authentication error if no API key is supplied for a remote provider. Set `REVIEWER_LLM_API_KEY` or add `api_key` to `reviewer.toml`.

## Unsupported Provider or Model
If the provider or model name is incorrect, the CLI exits with an error. Check the values in your configuration match a supported provider and model.

## No Files Reviewed
When `reviewlens` reports that no files were analyzed, verify that `paths.allow` includes the files you expect and that they exist in the repository.

## Network Errors
Connectivity problems can cause requests to fail. If you need an offline run, keep `provider = "null"` to use the built-in local mode.

## CI Failures
In CI environments ensure:
- The repository is checked out before running the CLI.
- Required secrets such as API keys are provided as environment variables.
- The exit code is handled to block merges when issues are found.

## Privacy Concerns
By default, only allow‑listed paths are scanned and sensitive content is redacted. If data appears in logs, review your allowlist and redaction patterns.
