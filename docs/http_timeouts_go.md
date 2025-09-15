# http-timeouts-go

Detects Go HTTP requests that omit timeouts, either by using the default client or a custom `http.Client` without a `Timeout` set.

## Recommendation

Always specify a timeout on HTTP clients or requests to avoid hanging connections.

## Configuration

Ensure Go files are included in the path allowlist (for example, `**/*.go`).

```toml
[rules.http-timeouts-go]
enabled = true
severity = "medium"
```

## Suppression

To skip this rule for a specific line, add:

```text
// reviewlens:ignore http-timeouts-go [reason]
```

The comment may appear on the same line or the one directly above. Any optional
reason provided will be recorded in the logs when the finding is suppressed.

