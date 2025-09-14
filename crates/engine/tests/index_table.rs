use engine::{
    config::{Config, IndexConfig},
    ReviewEngine,
};
use serde_json::json;
use std::fs;
use std::io::Write;
use tempfile::{tempdir, NamedTempFile};

fn build_index(docs: &[(&str, &str)]) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("create temp index");
    let documents: Vec<_> = docs
        .iter()
        .map(|(f, c)| json!({"filename": f, "content": c}))
        .collect();
    let data = json!({"documents": documents});
    file.write_all(data.to_string().as_bytes())
        .expect("write index");
    file.flush().expect("flush index");
    file
}

#[tokio::test]
async fn loads_index_from_index_table() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("lib.rs");
    fs::write(&file_path, "fn main() {\n}").unwrap();
    let path_str = file_path.to_str().unwrap();
    let diff = format!(
        "diff --git a/{p} b/{p}\n--- a/{p}\n+++ b/{p}\n@@ -0,0 +1 @@\n+fn main() {{}}\n",
        p = path_str
    );

    let index = build_index(&[("existing.rs", "fn existing() { log::info!(\\\"hi\\\"); }")]);
    let mut config = Config::default();
    config.index = Some(IndexConfig {
        path: index.path().to_str().unwrap().to_string(),
    });

    let engine = ReviewEngine::new(config).unwrap();
    let report = engine.run(&diff).await.unwrap();
    assert!(report.metadata.index_warm);
}
