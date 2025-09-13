use engine::rag::{index_repository, InMemoryVectorStore, RagContextRetriever, VectorStore};
use std::env;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::tempdir;

#[tokio::test]
async fn retrieves_context_from_saved_store() {
    // Prepare a store with a known document
    let mut store = InMemoryVectorStore::default();
    store
        .add("example context".to_string(), Vec::new())
        .await
        .unwrap();

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
    let index_path = dir.path().join("index.json");

    let store = index_repository(dir.path(), &index_path, false)
        .await
        .unwrap();

    assert_eq!(store.len(), 1);
    assert!(index_path.exists());
}

#[tokio::test]
async fn uses_cached_index_when_not_forced() {
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

    // Without force, the cached index should be used (still 1 document)
    let cached = index_repository(dir.path(), &index_path, false)
        .await
        .unwrap();
    assert_eq!(cached.len(), 1);

    // Forcing rebuild should pick up the new file
    let rebuilt = index_repository(dir.path(), &index_path, true)
        .await
        .unwrap();
    assert_eq!(rebuilt.len(), 2);
}
