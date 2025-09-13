//! Retrieval-Augmented Generation (RAG) components.
//!
//! This module provides the traits and structures for indexing a codebase
//! and retrieving relevant context to inform the LLM's analysis.

use crate::error::{EngineError, Result};
use async_trait::async_trait;
use std::fs;
use walkdir::WalkDir;

/// A trait for a vector store that can store and retrieve embeddings.
#[async_trait]
pub trait VectorStore {
    /// Adds a document and its vector embedding to the store.
    async fn add(&mut self, document: String, embedding: Vec<f32>) -> Result<()>;

    /// Searches for the most similar documents to a given query vector.
    async fn search(&self, query_embedding: Vec<f32>, top_k: usize) -> Result<Vec<String>>;
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
        // 1. Generate embedding for the query.
        let embedding: Vec<f32> = query.bytes().map(|b| b as f32).collect();

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
            .map(|(i, doc)| format!("{}. {}", i + 1, doc))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(formatted)
    }
}

/// A simple in-memory vector store for demonstration purposes.
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct InMemoryVectorStore {
    documents: Vec<String>,
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
    async fn add(&mut self, document: String, _embedding: Vec<f32>) -> Result<()> {
        self.documents.push(document);
        Ok(())
    }

    /// Returns up to `top_k` documents from the in-memory store.
    async fn search(&self, _query_embedding: Vec<f32>, top_k: usize) -> Result<Vec<String>> {
        Ok(self.documents.iter().take(top_k).cloned().collect())
    }
}

/// Indexes all files under `path` and populates an `InMemoryVectorStore`.
pub async fn index_repository(path: &str, force: bool) -> Result<InMemoryVectorStore> {
    log::info!("Indexing repository at {} (force={})", path, force);
    let mut store = InMemoryVectorStore::default();

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let content = fs::read_to_string(entry.path())?;
            let embedding: Vec<f32> = content.bytes().map(|b| b as f32).collect();
            store.add(content, embedding).await?;
        }
    }

    log::info!("Indexed {} files", store.len());
    Ok(store)
}
