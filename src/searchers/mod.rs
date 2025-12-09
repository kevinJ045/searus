//! Searcher implementations for the Searus search engine.

pub mod bm25;
pub mod fuzzy;
pub mod semantic;
pub mod tagged;
pub mod tokenizer;

pub use fuzzy::FuzzySearch;
pub use semantic::SemanticSearch;
pub use tagged::TaggedSearch;
