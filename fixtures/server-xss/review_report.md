# Code Review Report

## Summary

Reviewed 1 file and found 1 issue. Notable findings: Unescaped user input written to ResponseWriter in main.go:10

## 🚨 Security Findings

| Severity | Title | File:Line | Description | Suggested Fix |
|---|---|---|---|---|
| `Medium` | Unescaped user input written to ResponseWriter | `main.go:10` | Writing untrusted input directly to http.ResponseWriter can lead to XSS. | Escape user input before writing to the response. |

<details>
<summary>Diff suggestion for `Unescaped user input written to ResponseWriter` at `main.go:10`</summary>

```diff
-fmt.Fprintf(w, "<p>"+user+"</p>")
+// escape user input before writing
```
</details>

## 🧹 Code Quality & Conventions

No code quality issues found.

## 🔥 Hotspots

| File | Changes |
|---|---|
| `main.go` | risk 14 |

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
