//! A scanner for detecting secrets and credentials.

use once_cell::sync::Lazy;
use regex::Regex;

use crate::config::Config;
use crate::error::Result;
use crate::scanner::{Issue, Scanner};

pub struct SecretsScanner;

// A set of regexes to detect common secret patterns.
// Using `once_cell::sync::Lazy` for one-time compilation of regexes.
static SECRET_REGEXES: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Generic API Key
        Regex::new(r#"(?i)api[_-]?key\s*[:=]\s*['"][a-zA-Z0-9\-_]{16,}['"]"#).unwrap(),
        // AWS Secret Key
        Regex::new(r#"(?i)aws_secret_access_key\s*[:=]\s*['"][a-zA-Z0-9/+=]{40}['"]"#).unwrap(),
        // Generic Token
        Regex::new(r#"(?i)token\s*[:=]\s*['"][a-zA-Z0-9\-_]{20,}['"]"#).unwrap(),
        // Private Key
        Regex::new(r"-----BEGIN [A-Z ]+ PRIVATE KEY-----").unwrap(),
    ]
});

impl Scanner for SecretsScanner {
    fn name(&self) -> &'static str {
        "Secrets Scanner"
    }

    fn scan(&self, file_path: &str, content: &str, config: &Config) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();
        for (i, line) in content.lines().enumerate() {
            for regex in &*SECRET_REGEXES {
                if regex.is_match(line) {
                    issues.push(Issue {
                        title: "Potential Secret Found".to_string(),
                        description: format!(
                            "A line matching the pattern for a secret was found: `{}`. Please verify and rotate if necessary.",
                            regex.as_str()
                        ),
                        file_path: file_path.to_string(),
                        line_number: i + 1,
                        severity: config.rules.secrets.severity.clone(),
                        suggested_fix: Some("Remove secrets from source control and use secure storage or environment variables.".to_string()),
                        diff: Some(format!("-{}\n+<redacted>", line.trim())),
                    });
                    // Don't flag the same line multiple times
                    break;
                }
            }
        }
        Ok(issues)
    }
}
