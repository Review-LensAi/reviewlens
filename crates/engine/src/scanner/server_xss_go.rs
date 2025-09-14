use crate::config::Config;
use crate::error::Result;
use crate::scanner::{Issue, Scanner};
use once_cell::sync::Lazy;
use regex::Regex;

pub struct ServerXssGoScanner;

static TEXT_TEMPLATE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)text/template").unwrap());
static UNSAFE_WRITE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)fmt\.Fprintf\s*\(\s*w,[^)]*\+"#).unwrap());

impl Scanner for ServerXssGoScanner {
    fn name(&self) -> &'static str {
        "Server XSS Scanner (Go)"
    }

    fn scan(&self, file_path: &str, content: &str, config: &Config) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();
        for (i, line) in content.lines().enumerate() {
            if TEXT_TEMPLATE_REGEX.is_match(line) {
                issues.push(Issue {
                    title: "text/template used for HTML".to_string(),
                    description:
                        "text/template does not auto-escape HTML; use html/template instead."
                            .to_string(),
                    file_path: file_path.to_string(),
                    line_number: i + 1,
                    severity: config.rules.server_xss_go.severity.clone(),
                    suggested_fix: Some("Use html/template which auto-escapes HTML.".to_string()),
                    diff: Some(format!(
                        "-{}\n+{}",
                        line.trim(),
                        line.replace("text/template", "html/template").trim()
                    )),
                });
            }
            if UNSAFE_WRITE_REGEX.is_match(line)
                || (line.contains("w.Write(") && line.contains("FormValue("))
            {
                issues.push(Issue {
                    title: "Unescaped user input written to ResponseWriter".to_string(),
                    description:
                        "Writing untrusted input directly to http.ResponseWriter can lead to XSS."
                            .to_string(),
                    file_path: file_path.to_string(),
                    line_number: i + 1,
                    severity: config.rules.server_xss_go.severity.clone(),
                    suggested_fix: Some(
                        "Escape user input before writing to the response.".to_string(),
                    ),
                    diff: Some(format!(
                        "-{}\n+// escape user input before writing",
                        line.trim()
                    )),
                });
            }
        }
        Ok(issues)
    }
}
