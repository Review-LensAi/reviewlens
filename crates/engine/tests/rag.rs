use engine::rag::{InMemoryVectorStore, RagContextRetriever, VectorStore};
use std::env;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

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
