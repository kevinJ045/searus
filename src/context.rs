//! Context provided to searchers during a search operation.

use std::any::Any;
use std::collections::HashMap;

/// A context object that provides access to the items being searched and other shared resources.
///
/// The `SearchContext` is passed to every `Searcher` during a search operation. It allows
/// searchers to access the data they need to perform their search, such as the items themselves,
/// global statistics, or shared caches.
pub struct SearchContext<'a, T> {
  /// The slice of items to be searched.
  pub items: &'a [T],
  /// A map of shared resources or metadata that can be used by searchers.
  /// This allows for extensibility without modifying the `SearchContext` struct itself.
  /// For example, a searcher could store pre-computed statistics here to be shared
  /// across multiple calls or with other searchers.
  pub cache: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl<'a, T> SearchContext<'a, T> {
  /// Creates a new `SearchContext` with the given items.
  pub fn new(items: &'a [T]) -> Self {
    Self {
      items,
      cache: HashMap::new(),
    }
  }

  /// Adds a value to the context's cache.
  pub fn with_cache_value<V: Any + Send + Sync>(mut self, key: impl Into<String>, value: V) -> Self {
    self.cache.insert(key.into(), Box::new(value));
    self
  }

  /// Retrieves a value from the context's cache.
  pub fn get_cache_value<V: Any + 'static>(&self, key: &str) -> Option<&V> {
    self.cache.get(key).and_then(|v| v.downcast_ref::<V>())
  }
}
