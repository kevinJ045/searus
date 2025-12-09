//! In-memory index implementation.

use crate::index::adapter::IndexAdapter;
use crate::types::EntityId;
use std::collections::HashMap;

/// In-memory index using HashMaps.
pub struct InMemIndex<T: Send + Sync> {
    items: HashMap<EntityId, T>,
    vectors: HashMap<EntityId, Vec<f32>>,
    tags: HashMap<EntityId, Vec<String>>,
}

impl<T: Send + Sync> InMemIndex<T> {
    /// Create a new empty in-memory index.
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            vectors: HashMap::new(),
            tags: HashMap::new(),
        }
    }
}

impl<T: Send + Sync> Default for InMemIndex<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Send + Sync> IndexAdapter<T> for InMemIndex<T> {
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

    fn remove(&mut self, id: &EntityId) -> Result<(), String> {
        self.items.remove(id);
        self.vectors.remove(id);
        self.tags.remove(id);
        Ok(())
    }

    fn get(&self, id: &EntityId) -> Option<&T> {
        self.items.get(id)
    }

    fn knn(&self, vector: &[f32], k: usize) -> Vec<(EntityId, f32)> {
        // Brute-force nearest neighbor search
        let mut distances: Vec<(EntityId, f32)> = self
            .vectors
            .iter()
            .map(|(id, v)| {
                let dist = euclidean_distance(vector, v);
                (id.clone(), dist)
            })
            .collect();

        // Sort by distance (ascending)
        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top k
        distances.into_iter().take(k).collect()
    }

    fn all(&self) -> Vec<&T> {
        self.items.values().collect()
    }
}

/// Calculate Euclidean distance between two vectors.
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
