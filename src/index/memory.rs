//! An in-memory implementation of the `IndexAdapter` trait.

use crate::index::adapter::IndexAdapter;
use crate::types::EntityId;
use std::collections::HashMap;

/// An in-memory search index that stores data in `HashMap`s.
///
/// `InMemIndex` is a simple and fast index implementation that is useful for
/// testing, prototyping, or for applications with small to medium-sized datasets
/// that can comfortably fit in memory. It is not persistent and all data will
/// be lost when the index is dropped.
pub struct InMemIndex<T: Send + Sync> {
  /// Stores the actual items, keyed by their `EntityId`.
  items: HashMap<EntityId, T>,
  /// Stores vector embeddings, keyed by their `EntityId`.
  vectors: HashMap<EntityId, Vec<f32>>,
  /// Stores tags, keyed by their `EntityId`.
  tags: HashMap<EntityId, Vec<String>>,
}

impl<T: Send + Sync> InMemIndex<T> {
  /// Creates a new, empty `InMemIndex`.
  pub fn new() -> Self {
    Self {
      items: HashMap::new(),
      vectors: HashMap::new(),
      tags: HashMap::new(),
    }
  }
}

impl<T: Send + Sync> Default for InMemIndex<T> {
  /// Creates a new, empty `InMemIndex`.
  fn default() -> Self {
    Self::new()
  }
}

impl<T: Send + Sync> IndexAdapter<T> for InMemIndex<T> {
  /// Adds or updates an item in the index.
  fn put(
    &mut self,
    id: EntityId,
    item: T,
    vectors: Option<Vec<f32>>,
    tags: Option<Vec<String>>,
  ) -> Result<(), String> {
    self.items.insert(id.clone(), item);

    if let Some(v) = vectors {
      self.vectors.insert(id.clone(), v);
    }

    if let Some(t) = tags {
      self.tags.insert(id, t);
    }

    Ok(())
  }

  /// Removes an item from the index by its ID.
  fn remove(&mut self, id: &EntityId) -> Result<(), String> {
    self.items.remove(id);
    self.vectors.remove(id);
    self.tags.remove(id);
    Ok(())
  }

  /// Retrieves an item from the index by its ID.
  fn get(&self, id: &EntityId) -> Option<&T> {
    self.items.get(id)
  }

  /// Performs a k-nearest neighbors search using a brute-force approach.
  ///
  /// This implementation iterates through all vectors in the index, calculates
  /// the Euclidean distance to the query vector for each one, and then sorts
  /// them to find the `k` nearest neighbors.
  ///
  /// # Warning
  ///
  /// This is an O(n) operation and can be slow for large datasets. For
  /// production use with many vectors, a more optimized index structure
  /// (e.g., an HNSW index) is recommended.
  fn knn(&self, vector: &[f32], k: usize) -> Vec<(EntityId, f32)> {
    let mut distances: Vec<(EntityId, f32)> = self
      .vectors
      .iter()
      .map(|(id, v)| {
        let dist = euclidean_distance(vector, v);
        (id.clone(), dist)
      })
      .collect();

    // Sort by distance in ascending order.
    distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    // Return the top k results.
    distances.into_iter().take(k).collect()
  }

  /// Retrieves all items currently in the index.
  fn all(&self) -> Vec<&T> {
    self.items.values().collect()
  }
}

/// Calculates the Euclidean distance between two vectors (slices of f32).
///
/// Euclidean distance is the straight-line distance between two points in
/// Euclidean space.
fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
  if a.len() != b.len() {
    return f32::INFINITY;
  }

  a.iter()
    .zip(b.iter())
    .map(|(x, y)| (x - y).powi(2))
    .sum::<f32>()
    .sqrt()
}
