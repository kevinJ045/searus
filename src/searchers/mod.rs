//! A collection of built-in `Searcher` implementations.
//!
//! This module provides several ready-to-use searchers that can be plugged
//! into the `SearusEngine`.

/// Implements the BM25 relevance scoring algorithm.
pub mod bm25;
/// Implements a fuzzy (approximate) string searcher.
pub mod fuzzy;
/// Implements a semantic searcher that uses BM25.
pub mod semantic;
/// Implements a searcher for matching tags.
pub mod tagged;
/// Provides text tokenization utilities for searchers.
pub mod tokenizer;

pub use fuzzy::FuzzySearch;
pub use semantic::SemanticSearch;
pub use tagged::TaggedSearch;
