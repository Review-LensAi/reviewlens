# Code Review Report

## Summary

Reviewed 1 file and found 1 issue. Notable findings: Potential SQL Injection in main.go:6

## 🚨 Security Findings

| Severity | Title | File:Line | Description | Suggested Fix |
|---|---|---|---|---|
| `Medium` | Potential SQL Injection | `main.go:6` | Dynamic SQL query construction detected. Use parameterized queries instead. | Use parameterized queries instead of string concatenation. |

<details>
<summary>Diff suggestion for `Potential SQL Injection` at `main.go:6`</summary>

```diff
-query := "SELECT * FROM users WHERE name = '" + user + "'"
+db.Query("...", params)
```
</details>

## 🧹 Code Quality & Conventions

No code quality issues found.

## 🔥 Hotspots

| File | Changes |
|---|---|
| `main.go` | risk 11 |

---

## Appendix

### Run Metadata

```json
{
  "ruleset_version": "1.0.0",
  "driver": "null",
  "timings": {
    "total_ms": 5
  },
  "index_warm": true
}
```

### Configuration Snapshot

This review was run with the following configuration:

```json
{
  "llm": {
    "provider": "null"
  },
  "budget": {
    "[REDACTED]s": {}
  },
  "generation": {},
  "privacy": {
    "redaction": {
      "enabled": true,
      "patterns": [
        "(?i)api[_-]?key",
        "[REDACTED]",
        "[REDACTED]"
      ]
    }
  },
  "paths": {
    "allow": [
      "**/*"
    ],
    "deny": []
  },
  "report": {
    "hotspot-weights": {
      "severity": 3,
      "churn": 1
    }
  },
  "index": {
    "path": "index.json"
  },
  "rules": {
    "secrets": {
      "enabled": true,
      "severity": "medium"
    },
    "sql-injection-go": {
      "enabled": true,
      "severity": "medium"
    },
    "http-timeouts-go": {
      "enabled": true,
      "severity": "medium"
    },
    "convention-deviation": {
      "enabled": true,
      "severity": "medium"
    },
    "server-xss-go": {
      "enabled": true,
      "severity": "medium"
    }
  },
  "fail-on": "low"
}
```
