use engine::config::Config;
use engine::redact_text;

#[test]
fn redacts_configured_patterns() {
    let mut config = Config::default();
    config.privacy.redaction.patterns.push("secret".to_string());
    let input = "this has a secret value";
    let output = redact_text(&config, input);
    assert_eq!(output, "this has a [REDACTED] value");
}

#[test]
fn redacts_default_patterns() {
    let config = Config::default();
    let input = "API-KEY aws_secret_access_key token";
    let output = redact_text(&config, input);
    assert_eq!(output, "[REDACTED] [REDACTED] [REDACTED]");
}

#[test]
fn respects_disabled_redaction() {
    let mut config = Config::default();
    config.privacy.redaction.patterns.push("secret".to_string());
    config.privacy.redaction.enabled = false;
    let input = "this has a secret token";
    let output = redact_text(&config, input);
    assert_eq!(output, input);
}
