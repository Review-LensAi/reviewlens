# http-timeouts-go

Detects Go HTTP requests that omit timeouts, either by using the default client or a custom `http.Client` without a `Timeout` set.

## Recommendation

Always specify a timeout on HTTP clients or requests to avoid hanging connections.

## Configuration

```toml
[rules.http-timeouts-go]
enabled = true
severity = "medium"
```

