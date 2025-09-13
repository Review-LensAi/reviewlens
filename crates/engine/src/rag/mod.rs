//! Retrieval-Augmented Generation (RAG) components.
//!
//! This module provides the traits and structures for indexing a codebase
//! and retrieving relevant context to inform the LLM's analysis.

use crate::error::{EngineError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Represents a single indexed document along with minimal metadata.
#[derive(Clone, Serialize, Deserialize)]
pub struct Document {
    pub filename: String,
    pub content: String,
    pub token_count: usize,
}

/// A trait for a vector store that can store and retrieve embeddings.
#[async_trait]
pub trait VectorStore {
    /// Adds a document and its vector embedding to the store.
    async fn add(&mut self, document: Document, embedding: Vec<f32>) -> Result<()>;

    /// Searches for the most similar documents to a given query vector.
    async fn search(&self, query_embedding: Vec<f32>, top_k: usize) -> Result<Vec<Document>>;
}

/// A trait for an indexer that processes source code and populates a vector store.
#[async_trait]
pub trait Indexer {
    /// Indexes a collection of file paths.
    ///
    /// This method would typically read files, chunk them, generate embeddings,
    /// and add them to the associated `VectorStore`.
    async fn index_paths(&self, paths: &[String]) -> Result<()>;
}

// Example of a simple RAG context retriever.
pub struct RagContextRetriever {
    /// The vector store used to search for similar documents.
    ///
    /// In a real implementation this would likely be backed by an external
    /// service such as Qdrant or Tantivy. Here we keep the trait object to
    /// allow different store implementations.
    vector_store: Box<dyn VectorStore + Send + Sync>,
}

impl RagContextRetriever {
    /// Creates a new `RagContextRetriever` with the provided vector store.
    pub fn new(vector_store: Box<dyn VectorStore + Send + Sync>) -> Self {
        Self { vector_store }
    }

    pub async fn retrieve(&self, query: &str) -> Result<String> {
        log::debug!("Retrieving RAG context for query: {}", query);
        // 1. Generate a tiny embedding for the query using token counts.
        let token_count = query.split_whitespace().count() as f32;
        let embedding = vec![token_count];

        // 2. Search the vector store.
        let top_k = 5;
        let results = self
            .vector_store
            .search(embedding, top_k)
            .await
            .map_err(|e| EngineError::Rag(format!("Vector store search failed: {e}")))?;

        // 3. Format and return the results as a string.
        if results.is_empty() {
            return Err(EngineError::Rag("No results found".into()));
        }

        let formatted = results
            .into_iter()
            .enumerate()
            .map(|(i, doc)| format!("{}. {}: {}", i + 1, doc.filename, doc.content))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(formatted)
    }
}

/// A simple in-memory vector store for demonstration purposes.
#[derive(Default, Serialize, Deserialize)]
pub struct InMemoryVectorStore {
    documents: Vec<Document>,
}

impl InMemoryVectorStore {
    /// Returns the number of documents stored.
    pub fn len(&self) -> usize {
        self.documents.len()
    }
}

#[async_trait]
impl VectorStore for InMemoryVectorStore {
    /// Stores the document in memory. Embeddings are ignored in this simple example.
    async fn add(&mut self, document: Document, _embedding: Vec<f32>) -> Result<()> {
        self.documents.push(document);
        Ok(())
    }

    /// Returns up to `top_k` documents from the in-memory store.
    async fn search(&self, _query_embedding: Vec<f32>, top_k: usize) -> Result<Vec<Document>> {
        Ok(self.documents.iter().take(top_k).cloned().collect())
    }
}

impl InMemoryVectorStore {
    /// Saves the vector store to the given path in JSON format.
    pub fn save_to_disk<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let data = serde_json::to_vec(&self)
            .map_err(|e| EngineError::Rag(format!("Failed to serialize store: {e}")))?;
        fs::write(path, data)?;
        Ok(())
    }

    /// Loads the vector store from the given path. If the file does not
    /// exist or cannot be deserialized, an error is returned.
    pub fn load_from_disk<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = fs::read(path)?;
        serde_json::from_slice(&data)
            .map_err(|e| EngineError::Rag(format!("Failed to deserialize store: {e}")))
    }
}

/// Indexes all files under `path` and populates an `InMemoryVectorStore`.
///
/// If `force` is `false` and an index already exists at `output`, the existing
/// index is loaded from disk instead of re-indexing the repository. When a new
/// index is built, it is persisted to the given `output` path.
pub async fn index_repository<P, Q>(path: P, output: Q, force: bool) -> Result<InMemoryVectorStore>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let path_ref = path.as_ref();
    let output_ref = output.as_ref();
    log::info!(
        "Indexing repository at {} (force={})",
        path_ref.display(),
        force
    );

    if !force && output_ref.exists() {
        log::info!("Loading existing index from {}", output_ref.display());
        return InMemoryVectorStore::load_from_disk(output_ref);
    }

    let mut store = InMemoryVectorStore::default();

    for entry in WalkDir::new(path_ref).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let content = fs::read_to_string(entry.path())?;
            let tokens = content.split_whitespace().count();
            let doc = Document {
                filename: entry.path().display().to_string(),
                content,
                token_count: tokens,
            };
            store.add(doc, vec![tokens as f32]).await?;
        }
    }

    if let Some(parent) = output_ref.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    store.save_to_disk(output_ref)?;
    log::info!("Indexed {} files", store.len());
    Ok(store)
}
