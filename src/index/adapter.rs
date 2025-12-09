//! Defines the `IndexAdapter` trait for creating pluggable storage backends.

use crate::types::EntityId;

/// A trait that defines the common interface for a search index.
///
/// `IndexAdapter` provides an abstraction over the underlying storage and
/// retrieval mechanism for the search engine. This allows for different
/// backends to be used, such as an in-memory index for development or a
/// persistent, disk-based index for production.
///
/// The `Send` and `Sync` bounds are required to allow the index to be safely
/// shared across threads.
pub trait IndexAdapter<T>: Send + Sync {
  /// Adds or updates an item in the index.
  ///
  /// # Arguments
  ///
  /// * `id` - The unique `EntityId` for the item.
  /// * `item` - The item to be stored.
  /// * `vectors` - An optional vector embedding associated with the item.
  /// * `tags` - An optional list of tags associated with the item.
  ///
  /// # Returns
  ///
  /// A `Result` indicating success or failure.
  fn put(
    &mut self,
    id: EntityId,
    item: T,
    vectors: Option<Vec<f32>>,
    tags: Option<Vec<String>>,
  ) -> Result<(), String>;

  /// Removes an item from the index by its ID.
  ///
  /// # Arguments
  ///
  /// * `id` - The `EntityId` of the item to remove.
  ///
  /// # Returns
  ///
  /// A `Result` indicating success or failure.
  fn remove(&mut self, id: &EntityId) -> Result<(), String>;

  /// Retrieves an item from the index by its ID.
  ///
  /// # Arguments
  ///
  /// * `id` - The `EntityId` of the item to retrieve.
  ///
  /// # Returns
  ///
  /// An `Option` containing a reference to the item if it exists, or `None` otherwise.
  fn get(&self, id: &EntityId) -> Option<&T>;

  /// Performs a k-nearest neighbors (k-NN) search.
  ///
  /// This method finds the `k` items in the index whose vector embeddings are
  /// closest to the given query vector.
  ///
  /// # Arguments
  ///
  /// * `vector` - The query vector to find neighbors for.
  /// * `k` - The number of nearest neighbors to return.
  ///
  /// # Returns
  ///
  /// A `Vec` of tuples, where each tuple contains the `EntityId` of a neighbor
  /// and its distance from the query vector.
  fn knn(&self, vector: &[f32], k: usize) -> Vec<(EntityId, f32)>;

  /// Retrieves all items currently in the index.
  ///
  /// # Returns
  ///
  /// A `Vec` containing references to all items in the index.
  fn all(&self) -> Vec<&T>;
}
