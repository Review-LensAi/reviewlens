# secrets

Detects potential secrets and credentials committed to the repository, such as API keys, tokens, or private keys.

## Recommendation

Remove secrets from source control and store them in a dedicated secrets manager or environment variables. Rotate any exposed credentials.

## Configuration

```toml
[rules.secrets]
enabled = true
severity = "medium"
```

