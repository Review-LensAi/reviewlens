use engine::{config::Config, ReviewEngine};
use std::fs;
use tempfile::tempdir;

#[tokio::test]
async fn generates_sequence_diagram_when_files_interact() {
    let dir = tempdir().unwrap();
    let file_a = dir.path().join("a.rs");
    let file_b = dir.path().join("b.rs");
    let file_c = dir.path().join("c.rs");
    fs::write(&file_a, "use crate::b; fn a() { b::b(); }\n").unwrap();
    fs::write(&file_b, "use crate::c; fn b() { c::c(); }\n").unwrap();
    fs::write(&file_c, "use crate::a; fn c() { a::a(); }\n").unwrap();

    let diff = format!(
        "diff --git a/{a} b/{a}\n--- a/{a}\n+++ b/{a}\n@@ -0,0 +1,1 @@\n+use crate::b; fn a() {{ b::b(); }}\n\
diff --git a/{b} b/{b}\n--- a/{b}\n+++ b/{b}\n@@ -0,0 +1,1 @@\n+use crate::c; fn b() {{ c::c(); }}\n\
diff --git a/{c} b/{c}\n--- a/{c}\n+++ b/{c}\n@@ -0,0 +1,1 @@\n+use crate::a; fn c() {{ a::a(); }}\n",
        a = file_a.to_str().unwrap(),
        b = file_b.to_str().unwrap(),
        c = file_c.to_str().unwrap()
    );

    let engine = ReviewEngine::new(Config::default()).unwrap();
    let report = engine.run(&diff).await.unwrap();
    let diagram = report.mermaid_diagram.expect("expected diagram");
    assert!(diagram.contains("sequenceDiagram"));
    assert!(diagram.contains("a.rs->>b.rs"));
    assert!(diagram.contains("b.rs->>c.rs"));
    assert!(diagram.contains("c.rs->>a.rs"));
}
