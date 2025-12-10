//! A collection of built-in `Searcher` implementations.
//!
//! This module provides several ready-to-use searchers that can be plugged
//! into the `SearusEngine`.

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
