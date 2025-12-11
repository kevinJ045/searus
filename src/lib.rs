//! Searus - A flexible, multi-modal search engine library.
//!
//! Searus provides a unified interface for different search strategies including
//! semantic search, vector search, tag-based search, and fuzzy matching.

/// Defines the search context.
pub mod context;
/// Contains components for generating embeddings from data.
pub mod embeddings;
/// The core search engine component.
pub mod engine;
/// Defines the extension system.
pub mod extension;
/// Provides filtering capabilities for search results.
pub mod filter;
/// Defines the indexing structures for efficient search.
pub mod index;
/// Implements the rules for combining search results.
pub mod rules;
/// The main searcher trait and multi-searcher implementation.
pub mod searcher;
/// Contains various searcher implementations.
pub mod searchers;
/// Defines the core types used throughout the library.
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
