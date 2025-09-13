# convention-deviation

Detects code that deviates from conventions inferred from the existing repository, such as using `println!` instead of logging macros, calling `.unwrap()` or `.expect()`, or defining functions that do not return `Result` when all others do.

## Recommendation

Follow established repository patterns: prefer logging macros over `println!`, use proper error handling instead of `.unwrap()` or `.expect()`, and align function signatures with prevailing `Result`-based style.

## Configuration

```toml
[rules.convention-deviation]
enabled = true
severity = "medium"
```

