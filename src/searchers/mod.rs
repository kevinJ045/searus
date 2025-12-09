//! Searcher implementations for the Searus search engine.

pub mod tokenizer;
pub mod bm25;
pub mod semantic;
pub mod tagged;
pub mod fuzzy;

pub use semantic::SemanticSearch;
pub use tagged::TaggedSearch;
pub use fuzzy::FuzzySearch;
