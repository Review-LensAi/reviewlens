//! Retrieval-Augmented Generation (RAG) components.
//!
//! This module provides the traits and structures for indexing a codebase
//! and retrieving relevant context to inform the LLM's analysis.

use crate::error::{EngineError, Result};
use async_trait::async_trait;
use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::time::UNIX_EPOCH;
use walkdir::WalkDir;

const VCS_DIRS: [&str; 4] = [".git", ".hg", ".svn", ".bzr"];

/// Represents a single indexed document along with extracted metadata.
#[derive(Clone, Serialize, Deserialize)]
pub struct Document {
    /// Name of the file on disk.
    pub filename: String,
    /// Original file content. Stored to allow scanners to reason about
    /// conventions present in the repository.
    pub content: String,
    /// Lightweight n-gram embedding representing the file's content.
    ///
    /// This embedding is computed using a simple hashing trick to bucket
    /// token n-grams into a fixed-size vector. It enables inexpensive
    /// similarity search without relying on heavyweight language models.
    #[serde(default)]
    pub embedding: Vec<f32>,
    /// All function signatures discovered in this file.
    #[serde(default)]
    pub function_signatures: Vec<String>,
    /// Lines that contain logging macros such as `log::info!`.
    #[serde(default)]
    pub log_patterns: Vec<String>,
    /// Lines that contain common error-handling patterns (`unwrap`,
    /// `expect`, or `Result` usage).
    #[serde(default)]
    pub error_snippets: Vec<String>,
    /// Last modification time of the file in nanoseconds since Unix epoch.
    #[serde(default)]
    pub modified: u64,
}

/// Generate a simple n-gram embedding for the provided text.
///
/// The embedding is created by hashing each token bigram into a fixed-size
/// vector. The resulting vector is L1 normalised so that documents of
/// different lengths can still be compared.
fn ngram_embedding(text: &str) -> Vec<f32> {
    const N: usize = 2; // bigrams
    const DIM: usize = 128;
    let mut vec = vec![0f32; DIM];
    let tokens: Vec<&str> = text.split_whitespace().collect();
    if tokens.len() < N {
        return vec;
    }
    for i in 0..=tokens.len() - N {
        let ngram = tokens[i..i + N].join(" ");
        let mut hasher = DefaultHasher::new();
        ngram.hash(&mut hasher);
        let idx = (hasher.finish() as usize) % DIM;
        vec[idx] += 1.0;
    }
    let sum: f32 = vec.iter().sum();
    if sum > 0.0 {
        for v in &mut vec {
            *v /= sum;
        }
    }
    vec
}

fn extract_function_signatures(content: &str) -> Vec<String> {
    let re = Regex::new(r"(?m)^\s*fn\s+\w+[^\n]*").unwrap();
    re.find_iter(content)
        .map(|m| m.as_str().trim().to_string())
        .collect()
}

fn extract_log_patterns(content: &str) -> Vec<String> {
    content
        .lines()
        .filter(|line| {
            line.contains("log::") || line.contains("println!") || line.contains("eprintln!")
        })
        .map(|l| l.trim().to_string())
        .collect()
}

fn extract_error_snippets(content: &str) -> Vec<String> {
    content
        .lines()
        .filter(|line| {
            line.contains(".unwrap()")
                || line.contains(".expect(")
                || line.contains("Result<")
                || line.contains("Err(")
        })
        .map(|l| l.trim().to_string())
        .collect()
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;
    for (x, y) in a.iter().zip(b) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a.sqrt() * norm_b.sqrt())
}

/// A trait for a vector store that can store and retrieve embeddings.
#[async_trait]
pub trait VectorStore {
    /// Adds a document (which already contains its embedding) to the store.
    async fn add(&mut self, document: Document) -> Result<()>;

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
        // 1. Generate a lightweight embedding for the query.
        let embedding = ngram_embedding(query);

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
    /// Stores the document in memory along with its embedding.
    async fn add(&mut self, document: Document) -> Result<()> {
        self.documents.push(document);
        Ok(())
    }

    /// Performs a naive cosine similarity search over stored embeddings.
    async fn search(&self, query_embedding: Vec<f32>, top_k: usize) -> Result<Vec<Document>> {
        let mut scored: Vec<(f32, Document)> = self
            .documents
            .iter()
            .cloned()
            .map(|doc| {
                let score = cosine_similarity(&query_embedding, &doc.embedding);
                (score, doc)
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        Ok(scored.into_iter().take(top_k).map(|(_, d)| d).collect())
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
/// Files are filtered using the provided allow and deny glob patterns. Paths
/// matching any deny pattern or not matching any allow pattern are skipped.
/// Version control directories such as `.git` are ignored automatically.
///
/// If `force` is `false` and an index already exists at `output`, the existing
/// index is loaded from disk and only files whose modification times have
/// changed are re-processed. When a new or updated index is built, it is
/// persisted to the given `output` path.
pub async fn index_repository<P, Q>(
    path: P,
    output: Q,
    force: bool,
    allow: &[String],
    deny: &[String],
) -> Result<InMemoryVectorStore>
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

    let allow_set = build_globset(allow)?;
    let deny_set = build_globset(deny)?;

    let mut store = if !force && output_ref.exists() {
        log::info!("Loading existing index from {}", output_ref.display());
        InMemoryVectorStore::load_from_disk(output_ref)?
    } else {
        InMemoryVectorStore::default()
    };

    let mut existing = std::mem::take(&mut store.documents)
        .into_iter()
        .map(|d| (d.filename.clone(), d))
        .collect::<HashMap<_, _>>();

    let mut new_documents = Vec::new();

    for entry in WalkDir::new(path_ref)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !VCS_DIRS.contains(&name.as_ref())
            } else {
                true
            }
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            let rel_path = entry.path().strip_prefix(path_ref).unwrap_or(entry.path());
            if !(allow_set.is_match(rel_path) && !deny_set.is_match(rel_path)) {
                continue;
            }
            let filename = rel_path.display().to_string();
            let modified_time = fs::metadata(entry.path())?
                .modified()?
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default();
            let modified =
                modified_time.as_secs() * 1_000_000_000 + u64::from(modified_time.subsec_nanos());

            if !force {
                if let Some(doc) = existing.get(&filename) {
                    if doc.modified == modified {
                        new_documents.push(doc.clone());
                        existing.remove(&filename);
                        continue;
                    }
                }
            }

            let content = fs::read_to_string(entry.path())?;
            let embedding = ngram_embedding(&content);
            let function_signatures = extract_function_signatures(&content);
            let log_patterns = extract_log_patterns(&content);
            let error_snippets = extract_error_snippets(&content);
            let doc = Document {
                filename: filename.clone(),
                content,
                embedding,
                function_signatures,
                log_patterns,
                error_snippets,
                modified,
            };
            new_documents.push(doc);
        }
    }

    store.documents = new_documents;

    if let Some(parent) = output_ref.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    store.save_to_disk(output_ref)?;
    log::info!("Indexed {} files", store.len());
    Ok(store)
}

fn build_globset(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = Glob::new(pattern).map_err(|e| EngineError::Config(e.to_string()))?;
        builder.add(glob);
    }
    builder
        .build()
        .map_err(|e| EngineError::Config(e.to_string()))
}
