use engine::rag::{
    index_repository, Document, InMemoryVectorStore, RagContextRetriever, VectorStore,
};
use std::env;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::tempdir;

#[tokio::test]
async fn retrieves_context_from_saved_store() {
    // Prepare a store with a known document
    let mut store = InMemoryVectorStore::default();
    let doc = Document {
        filename: "doc.txt".into(),
        content: "example context".into(),
        embedding: vec![1.0; 128],
        function_signatures: vec![],
        log_patterns: vec![],
        error_snippets: vec![],
        modified: 0,
    };
    store.add(doc).await.unwrap();

    // Persist the store to disk
    let mut path = env::temp_dir();
    let filename = format!(
        "vector_store_{}.json",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    path.push(filename);
    store.save_to_disk(&path).unwrap();

    // Load it back and ensure retrieval works
    let loaded = InMemoryVectorStore::load_from_disk(&path).unwrap();
    fs::remove_file(&path).unwrap();

    let rag = RagContextRetriever::new(Box::new(loaded));
    let ctx = rag.retrieve("whatever").await.unwrap();
    assert!(ctx.contains("example context"));
}

#[tokio::test]
async fn indexes_repository_and_saves_to_disk() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("file.txt");
    fs::write(&file_path, "content").unwrap();
    let index_dir = tempdir().unwrap();
    let index_path = index_dir.path().join("index.json");

    let store = index_repository(dir.path(), &index_path, false)
        .await
        .unwrap();

    assert_eq!(store.len(), 1);
    assert!(index_path.exists());
}

#[tokio::test]
async fn updates_index_incrementally() {
    let dir = tempdir().unwrap();
    let file_a = dir.path().join("a.txt");
    fs::write(&file_a, "a").unwrap();
    let index_dir = tempdir().unwrap();
    let index_path = index_dir.path().join("index.json");

    // Initial indexing creates the cache
    let initial = index_repository(dir.path(), &index_path, false)
        .await
        .unwrap();
    assert_eq!(initial.len(), 1);

    // Add another file after the cache exists
    let file_b = dir.path().join("b.txt");
    fs::write(&file_b, "b").unwrap();

    // Re-index without force should pick up the new file
    let updated = index_repository(dir.path(), &index_path, false)
        .await
        .unwrap();
    assert_eq!(updated.len(), 2);

    // Modify an existing file and ensure the content is refreshed
    fs::write(&file_a, "a changed").unwrap();
    let refreshed = index_repository(dir.path(), &index_path, false)
        .await
        .unwrap();
    assert_eq!(refreshed.len(), 2);
    let json = fs::read_to_string(&index_path).unwrap();
    assert!(json.contains("a changed"));

    // Forcing rebuild should produce the same result
    let rebuilt = index_repository(dir.path(), &index_path, true)
        .await
        .unwrap();
    assert_eq!(rebuilt.len(), 2);
}
