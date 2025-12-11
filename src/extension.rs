//! Defines the extension system for Searus.

use crate::searcher::Searcher;
use crate::types::{Query, Searchable, SearusMatch};

/// A trait for extensions that can hook into the search lifecycle.
///
/// Extensions allow for modifying queries, items, and results at various stages
/// of the search process. They can be used for caching, query rewriting,
/// data fetching, filtering, and more.
///
/// # Examples
///
/// Implementing a simple logging extension:
///
/// ```rust
/// use searus::prelude::*;
///
/// struct LoggingExtension;
///
/// impl<T: Searchable> SearusExtension<T> for LoggingExtension {
///     fn before_query(&self, query: &mut Query) {
///         if let Some(text) = &query.text {
///             println!("Processing query: {}", text);
///         }
///     }
///
///     fn after_limit(&self, _query: &Query, results: &mut Vec<SearusMatch<T>>) {
///         println!("Returning {} results", results.len());
///     }
/// }
/// ```
pub trait SearusExtension<T: Searchable>: Send + Sync {
  /// Called before the query is processed.
  ///
  /// This hook allows modifying the query before it is used for search.
  /// For example, a query rewriter extension could expand terms or add filters.
  ///
  /// # Examples
  ///
  /// ```rust
  /// # use searus::prelude::*;
  /// # struct MyExt;
  /// # impl<T: Searchable> SearusExtension<T> for MyExt {
  /// fn before_query(&self, query: &mut Query) {
  ///     // Force all queries to be lowercase
  ///     if let Some(text) = &mut query.text {
  ///         *text = text.to_lowercase();
  ///     }
  /// }
  /// # }
  /// ```
  fn before_query(&self, _query: &mut Query) {}

  /// Called before the items are passed to the searchers.
  ///
  /// This hook allows modifying the list of items to be searched.
  /// For example, an extension could fetch additional items from an external source
  /// or filter out items based on permissions.
  fn before_items(&self, _query: &Query, _items: &mut Vec<T>) {}

  /// Called before a specific searcher is executed.
  ///
  /// This hook allows inspecting or modifying the searcher before it runs.
  /// Note: Replacing the searcher is not directly supported via this hook in this signature,
  /// but internal state of the searcher could potentially be modified if `Searcher` exposed mutability,
  /// which it currently doesn't (it's `&self` in `search`).
  /// So this hook is mostly for side effects or logging in the current design,
  /// unless we change `Searcher` to be mutable or `Box<dyn Searcher>` to be mutable here.
  fn before_searcher(&self, _query: &Query, _searcher: &mut Box<dyn Searcher<T>>) {}

  /// Called after a specific searcher has executed.
  ///
  /// This hook allows modifying the raw results returned by a searcher.
  fn after_searcher(&self, _query: &Query, _results: &mut Vec<SearusMatch<T>>) {}

  /// Called before the results from all searchers are merged.
  ///
  /// This hook allows modifying the collection of all results before they are combined.
  /// The results are passed as a mutable vector of matches, which is what `merge_results` expects
  /// if we change the engine to flatten them first, or we can pass the structure `Vec<(SearcherKind, Vec<SearusMatch<T>>)>`.
  fn before_merge(
    &self,
    _query: &Query,
    _results: &mut Vec<(crate::types::SearcherKind, Vec<SearusMatch<T>>)>,
  ) {
  }

  /// Called after the results have been merged.
  ///
  /// This hook allows modifying the merged and scored results.
  fn after_merge(&self, _query: &Query, _results: &mut Vec<SearusMatch<T>>) {}

  /// Called before pagination (skip/limit) is applied.
  ///
  /// This is a good place for final sorting or filtering.
  fn before_limit(&self, _query: &Query, _results: &mut Vec<SearusMatch<T>>) {}

  /// Called after pagination is applied.
  ///
  /// This hook allows modifying the final set of results that will be returned to the user.
  fn after_limit(&self, _query: &Query, _results: &mut Vec<SearusMatch<T>>) {}
}
