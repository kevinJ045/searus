//! The Searcher trait and related types.

use crate::types::{Query, SearcherKind, SearusMatch};

/// A searcher plugin that performs search over items or an index.
pub trait Searcher<T>: Send + Sync {
  /// Returns the kind of this searcher.
  fn kind(&self) -> SearcherKind;

  /// Search over a slice of items.
  ///
  /// Implementations may ignore query fields they don't support.
  /// Returns matches with raw scores (not yet normalized).
  fn search(&self, query: &Query, items: &[T]) -> Vec<SearusMatch<T>>;
}
