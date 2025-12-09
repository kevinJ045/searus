//! Searus - A flexible, multi-modal search engine library.
//!
//! Searus provides a unified interface for different search strategies including
//! semantic search, vector search, tag-based search, and fuzzy matching.

pub mod types;
pub mod searcher;
pub mod engine;
pub mod rules;
pub mod filter;
pub mod embeddings;
pub mod index;
pub mod searchers;

pub mod prelude {
    //! Convenient re-exports for common types and traits.
    
    pub use crate::types::*;
    pub use crate::searcher::*;
    pub use crate::engine::*;
    pub use crate::rules::*;
    pub use crate::filter::*;
    pub use crate::embeddings::*;
    pub use crate::index::*;
    pub use crate::searchers::*;
}
