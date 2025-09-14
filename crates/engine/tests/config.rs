use engine::config::{Config, Provider, Severity};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fs};

#[test]
fn load_from_path_reads_new_toml_format() {
    let toml = r#"
[llm]
provider = "null"
model = "test-model"

[paths]
allow = ["src/**"]
deny = ["vendor/**"]

[privacy.redaction]
enabled = false

[rules.secrets]
enabled = true
severity = "critical"
"#;

    let mut path = env::temp_dir();
    let filename = format!(
        "reviewlens_test_{}.toml",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    path.push(filename);
    fs::write(&path, toml).unwrap();

    let config = Config::load_from_path(&path).expect("config should load");
    fs::remove_file(&path).unwrap();

    assert_eq!(config.llm.provider, Provider::Null);
    assert_eq!(config.llm.model, Some("test-model".to_string()));
    assert_eq!(config.paths.allow, vec!["src/**".to_string()]);
    assert_eq!(config.paths.deny, vec!["vendor/**".to_string()]);
    assert!(!config.privacy.redaction.enabled);
    assert!(config.rules.secrets.enabled);
    assert_eq!(config.rules.secrets.severity, Severity::Critical);
}

#[test]
fn default_config_is_sane() {
    let config = Config::default();
    assert_eq!(config.llm.provider, Provider::Null);
    assert!(config.privacy.redaction.enabled); // Should be true by default
    assert_eq!(
        config.privacy.redaction.patterns,
        vec![
            "(?i)api[_-]?key".to_string(),
            "aws_secret_access_key".to_string(),
            "token".to_string(),
        ]
    );
    assert!(config.rules.secrets.enabled);
    assert_eq!(config.rules.secrets.severity, Severity::High);
}
