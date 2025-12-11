//! Defines the extension system for Searus.

use crate::searcher::Searcher;
use crate::types::{Query, Searchable, SearusMatch};

/// A trait for extensions that can hook into the search lifecycle.
///
/// Extensions allow for modifying queries, items, and results at various stages
/// of the search process. They can be used for caching, query rewriting,
/// data fetching, filtering, and more.
pub trait SearusExtension<T: Searchable>: Send + Sync {
  /// Called before the query is processed.
  ///
  /// This hook allows modifying the query before it is used for search.
  /// For example, a query rewriter extension could expand terms or add filters.
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
  /// The `EXT.md` proposed `searcher: &mut Box<dyn Searcher<T>>`.
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
  /// `EXT.md` signature: `fn before_merge<T>(&self, query: &Query, results: &mut Vec<SearusMatch<T>>)`.
  /// This implies `before_merge` happens *after* flattening but *before* the logic that combines scores?
  /// Or maybe it means "before the final merge step".
  /// Let's look at `engine.rs`. `merge_results` takes `Vec<(SearcherKind, Vec<SearusMatch<T>>)>`.
  /// If `before_merge` takes `Vec<SearusMatch<T>>`, it implies the results are already flattened?
  /// Actually, `EXT.md` says "Useful for weighting searchers...". Weighting usually happens during merge.
  /// If we want to support weighting adjustments, we probably want access to the unmerged results.
  /// However, to stick to the `EXT.md` signature which uses `Vec<SearusMatch<T>>`, let's assume
  /// it runs *after* merge but *before* sorting/pagination?
  /// Wait, `EXT.md` lists:
  /// 5. before_merge
  /// 6. after_merge
  /// 7. before_limit
  ///
  /// If `before_merge` takes `Vec<SearusMatch<T>>`, then the merge must have already happened?
  /// That would make `before_merge` and `after_merge` redundant if they both take `Vec<SearusMatch<T>>`.
  /// Let's re-read `EXT.md`: "Useful for weighting searchers, boosting certain types, etc."
  /// This suggests `before_merge` should probably take the *unmerged* results: `&mut Vec<(SearcherKind, Vec<SearusMatch<T>>)>`.
  /// But the trait definition in `EXT.md` showed `results: &mut Vec<SearusMatch<T>>`.
  /// This might be a typo or simplification in `EXT.md`.
  /// Given the description, I will implement `before_merge` to take `&mut Vec<(SearcherKind, Vec<SearusMatch<T>>)>`
  /// to allow for weighting/boosting logic that depends on the searcher kind.
  /// And `after_merge` will take `&mut Vec<SearusMatch<T>>`.
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
