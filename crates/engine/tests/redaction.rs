use engine::config::Config;
use engine::redact_text;

#[test]
fn redacts_configured_patterns() {
    let mut config = Config::default();
    config.privacy.redaction.patterns.push("secret".to_string());
    let input = "this has a secret token";
    let output = redact_text(&config, input);
    assert_eq!(output, "this has a [REDACTED] token");
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
