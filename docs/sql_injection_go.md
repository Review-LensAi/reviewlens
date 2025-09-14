# sql-injection-go

Detects dynamic SQL query construction in Go code that could lead to SQL injection vulnerabilities.

## Recommendation

Use parameterized queries or prepared statements with the `database/sql` package instead of string concatenation to build queries.

## Configuration

```toml
[rules.sql-injection-go]
enabled = true
severity = "medium"
```

## Suppression

Use the following directive to ignore a specific finding:

```text
// reviewlens:ignore sql-injection-go [reason]
```

Add the comment on the offending line or the line above it. The optional reason
will be logged when the finding is suppressed.

## Testing notes

When writing tests for this rule, construct a `Config` that explicitly enables only
`sql-injection-go`. Relying on `Config::default()` can pull in other rules and lead to
non-deterministic behaviour if those rules modify shared state during testing.