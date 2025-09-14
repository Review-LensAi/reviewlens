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

## Suppression

To suppress a finding from this rule, add an inline comment:

```text
// reviewlens:ignore secrets [reason]
```

Place the directive on the same line as the code triggering the rule or on the
line immediately above it. An optional reason may be provided and will be
logged when the finding is ignored.

