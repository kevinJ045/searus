//! Provides abstractions and implementations for indexing and storing data.
//!
//! The `index` module defines a common interface for search indexes and provides
//! concrete implementations, such as an in-memory index.
//!
//! # Examples
//!
//! ```rust
//! use searus::index::{InMemIndex, IndexAdapter};
//!
//! let mut index = InMemIndex::new();
//! index.put("1".to_string(), "Document 1".to_string(), None, None).unwrap();
//! ```

/// Defines the `IndexAdapter` trait, the core abstraction for an index.
pub mod adapter;
/// Provides an in-memory implementation of the `IndexAdapter`.
pub mod memory;

pub use adapter::IndexAdapter;
pub use memory::InMemIndex;
