//! Index adapter trait for pluggable storage backends.

use searus_core::types::EntityId;

/// Trait for index storage backends.
pub trait IndexAdapter<T>: Send + Sync {
    /// Store an item with optional vectors and tags.
    fn put(
        &mut self,
        id: EntityId,
        item: T,
        vectors: Option<Vec<f32>>,
        tags: Option<Vec<String>>,
    ) -> Result<(), String>;

    /// Remove an item by ID.
    fn remove(&mut self, id: &EntityId) -> Result<(), String>;

    /// Get an item by ID.
    fn get(&self, id: &EntityId) -> Option<&T>;

    /// Find k nearest neighbors for a vector.
    fn knn(&self, vector: &[f32], k: usize) -> Vec<(EntityId, f32)>;

    /// Get all items.
    fn all(&self) -> Vec<&T>;
}
