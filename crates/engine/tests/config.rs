use engine::config::Config;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fs};

#[test]
fn load_from_path_reads_toml() {
    let toml = r#"
[llm]
provider = "local"
model = "dummy"
temperature = 0.5

[project]
include = ["src/**/*"]
exclude = ["target/*"]

[rules]
owasp-top-5 = true
secrets = false
"#;

    let mut path = env::temp_dir();
    let filename = format!("reviewlens_test_{}.toml", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos());
    path.push(filename);
    fs::write(&path, toml).unwrap();

    let config = Config::load_from_path(&path).expect("config should load");
    fs::remove_file(&path).unwrap();

    assert_eq!(config.llm.provider, "local");
    assert_eq!(config.project.include, vec!["src/**/*".to_string()]);
    assert!(config.rules.owasp_top_5);
    assert!(!config.rules.secrets);
}
