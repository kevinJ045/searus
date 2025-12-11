//! # Searus: A Flexible, Multi-Modal Search Engine Library for Rust
//!
//! Searus is a powerful, adaptable search library designed to provide a unified interface for various search strategies. Whether you need full-text search, semantic understanding, tag-based filtering, or fuzzy matching, Searus offers a cohesive solution. It is particularly well-suited for applications that require combining multiple search modalities to deliver nuanced and relevant results.
//!
//! The library is built with flexibility in mind, allowing you to compose different searchers, define custom scoring rules, and extend functionality with hooks into the search lifecycle.
//!
//! ## Key Features
//!
//! - **Multi-Modal Search**: Combine different searchers (e.g., `SemanticSearch`, `TaggedSearch`, `FuzzySearch`) in a single query.
//! - **Configurable Ranking**: Use `SemanticRules` to define field-specific weights, priorities, and search methods (like BM25 or exact matching).
//! - **Extensible Architecture**: Implement custom `Searcher` traits or use `SearusExtension` to modify queries and results.
//! - **Filtering**: Apply complex, field-based filters to refine search results before or after the search process.
//! - **Tag Relationship Trees (TRT)**: Expand tag-based queries to include related tags, enabling more comprehensive searches.
//! - **Parallel Execution**: Speed up searches with the optional `parallel` feature flag.
//!
//! ## Feature Flags
//!
//! - `semantic` (default): Enables semantic search capabilities (BM25).
//! - `fuzzy` (default): Enables fuzzy search capabilities.
//! - `tagged` (default): Enables tag-based search capabilities.
//! - `parallel`: Enables parallel execution using `rayon`.
//! - `serde`: Enables serialization support (required for most features).
//!
//! ## Getting Started
//!
//! Here's a quick example of how to set up a semantic search engine for a collection of blog posts.
//!
//! First, add Searus to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! searus = "0.1.0" # Replace with the latest version
//! serde = { version = "1.0", features = ["derive"] }
//! ```
//!
//! Now, you can create a search engine and query your data:
//!
//! ```rust
//! use searus::prelude::*;
//! use searus::searchers::SemanticSearch;
//! use serde::{Deserialize, Serialize};
//!
//! // Define the data structure to be searched.
//! // It must derive `Serialize`, `Deserialize`, and `Clone`.
//! #[derive(Debug, Clone, Serialize, Deserialize)]
//! struct Post {
//!     id: u32,
//!     title: String,
//!     content: String,
//!     author: String,
//! }
//!
//! fn main() {
//!     // 1. Create a collection of documents to search.
//!     let posts = vec![
//!         Post {
//!             id: 1,
//!             title: "Getting Started with Rust".to_string(),
//!             content: "Rust is a systems programming language.".to_string(),
//!             author: "Alice".to_string(),
//!         },
//!         Post {
//!             id: 2,
//!             title: "Building a Search Engine in Rust".to_string(),
//!             content: "Learn how to build a search engine.".to_string(),
//!             author: "Bob".to_string(),
//!         },
//!     ];
//!
//!     // 2. Define semantic rules for searching fields.
//!     // Here, we prioritize matches in 'title' over 'content'.
//!     let rules = SemanticRules::builder()
//!         .field("title", FieldRule::bm25().priority(2))
//!         .field("content", FieldRule::bm25().priority(1))
//!         .build();
//!
//!     // 3. Create a searcher. `SemanticSearch` is great for text-based queries.
//!     let semantic_searcher = SemanticSearch::new(rules);
//!
//!     // 4. Build the search engine and register the searcher.
//!     let engine = SearusEngine::builder()
//!         .with(Box::new(semantic_searcher))
//!         .build();
//!
//!     // 5. Construct a query.
//!     let query = Query::builder()
//!         .text("rust programming")
//!         .options(SearchOptions::default().limit(1))
//!         .build();
//!
//!     // 6. Execute the search.
//!     let results = engine.search(&posts, &query);
//!
//!     // 7. Print the results.
//!     println!("Query: \"rust programming\"");
//!     for result in results {
//!         println!(
//!             "Found post: \"{}\" by {} (Score: {:.3})",
//!             result.item.title, result.item.author, result.score
//!         );
//!     }
//! }
//! ```
//!
//! This example demonstrates the basic workflow: defining data, configuring rules, building an engine, and executing a query. For more advanced use cases, such as combining multiple searchers or using filters, see the documentation for `SearusEngine`, `Query`, and the specific `Searcher` implementations.

/// Provides the `SearchContext`, which holds the state of the items being searched.
pub mod context;
/// Contains components for generating embeddings, used in vector or semantic search.
/// (Currently experimental).
pub mod embeddings;
/// The core `SearusEngine`, which orchestrates the search process across multiple searchers.
pub mod engine;
/// Defines the `SearusExtension` trait for hooking into the search lifecycle to modify queries or results.
pub mod extension;
/// Provides powerful filtering capabilities with `FilterExpr` to refine search results.
pub mod filter;
/// Defines indexing structures for optimizing search performance.
/// (Currently includes in-memory adapters).
pub mod index;
/// Implements the `SemanticRules` and `FieldRule` for fine-grained control over text-based searching.
pub mod rules;
/// Contains the fundamental `Searcher` trait and the multi-searcher implementation.
pub mod searcher;
/// A collection of built-in `Searcher` implementations, including `SemanticSearch`, `TaggedSearch`, and `FuzzySearch`.
pub mod searchers;
/// Defines the core data structures used throughout the library, such as `Query`, `SearusMatch`, and `SearchOptions`.
pub mod types;

pub mod prelude {
  //! Convenient re-exports for common types and traits.

  pub use crate::context::*;
  pub use crate::embeddings::*;
  pub use crate::engine::*;
  pub use crate::extension::*;
  pub use crate::filter::*;
  pub use crate::index::*;
  pub use crate::rules::*;
  pub use crate::searcher::*;
  pub use crate::searchers::*;
  pub use crate::types::*;
}
