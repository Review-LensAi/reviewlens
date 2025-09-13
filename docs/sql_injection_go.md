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

