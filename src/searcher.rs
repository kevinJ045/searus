//! The `Searcher` trait, which defines the interface for search plugins.

use crate::types::{Query, SearcherKind, SearusMatch};

/// A trait for searcher plugins that can perform a search operation.
///
/// A `Searcher` is a modular component responsible for a specific type of search,
/// such as semantic search, tag-based search, or fuzzy matching. The `SearusEngine`
/// uses implementations of this trait to execute different search strategies in
/// parallel and then merges their results.
///
/// The `Send` and `Sync` bounds are required to allow searchers to be used
/// concurrently by the engine.
pub trait Searcher<T>: Send + Sync {
  /// Returns the `SearcherKind` of this searcher.
  ///
  /// This is used by the `SearusEngine` to identify the searcher and apply
  /// kind-specific configurations, such as weights.
  fn kind(&self) -> SearcherKind;

  /// Performs a search over a slice of items based on a query.
  ///
  /// # Arguments
  ///
  /// * `query` - The `Query` object containing the search parameters. Each
  ///   searcher implementation is responsible for extracting the parts of the
  ///   query that are relevant to it (e.g., a semantic searcher would use
  ///   `query.text`, while a tag searcher would use `query.tags`).
  /// * `items` - A slice of items to search through.
  ///
  /// # Returns
  ///
  /// A `Vec<SearusMatch<T>>` containing the matches found by this searcher.
  /// The scores in these matches are expected to be "raw" scores, meaning they
  /// have not yet been normalized. The `SearusEngine` will handle normalization
  /// before merging results.
  fn search(&self, query: &Query, items: &[T]) -> Vec<SearusMatch<T>>;
}
