//! A collection of built-in `Searcher` implementations.
//!
//! This module provides several ready-to-use searchers that can be plugged
//! into the `SearusEngine`.
//!
//! # Available Searchers
//!
//! - [`SemanticSearch`](crate::searchers::SemanticSearch): Best for natural language queries. Uses BM25 and tokenization.
//! - [`TaggedSearch`](crate::searchers::TaggedSearch): Best for exact tag matching and hierarchical tag expansion.
//! - [`FuzzySearch`](crate::searchers::FuzzySearch): Best for handling typos and approximate string matching.
//!
//! # Example: Combining Searchers
//!
//! ```rust
//! use searus::prelude::*;
//! use searus::searchers::{SemanticSearch, TaggedSearch};
//!
//! #[derive(Debug, Clone, serde::Serialize)]
//! struct Item {
//!     title: String,
//!     tags: Vec<String>,
//! }
//!
//! // Configure semantic search
//! let rules = SemanticRules::builder()
//!     .field("title", FieldRule::bm25())
//!     .build();
//! let semantic = SemanticSearch::new(rules);
//!
//! // Configure tag search
//! let tagged = TaggedSearch::new();
//!
//! // Combine in engine
//! let engine: SearusEngine<Item> = SearusEngine::builder()
//!     .with(Box::new(semantic))
//!     .with(Box::new(tagged))
//!     .build();
//! ```

/// Implements the BM25 relevance scoring algorithm.
#[cfg(feature = "semantic")]
pub mod bm25;
/// Implements a fuzzy (approximate) string searcher.
#[cfg(feature = "fuzzy")]
pub mod fuzzy;
/// Implements a semantic searcher that uses BM25.
#[cfg(feature = "semantic")]
pub mod semantic;
/// Implements a searcher for matching tags.
#[cfg(feature = "tagged")]
pub mod tagged;
/// Provides text tokenization utilities for searchers.
#[cfg(any(feature = "semantic", feature = "fuzzy"))]
pub mod tokenizer;

#[cfg(feature = "fuzzy")]
pub use fuzzy::FuzzySearch;
#[cfg(feature = "semantic")]
pub use semantic::SemanticSearch;
#[cfg(feature = "tagged")]
pub use tagged::TaggedSearch;
