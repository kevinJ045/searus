//! Embedding provider abstractions for Searus.
//! 
//! This crate provides traits and implementations for generating embeddings
//! from text and images. Embeddings are used for vector-based search.

/// Trait for text embedding providers.
pub trait TextEmbedder: Send + Sync {
    /// Generate an embedding vector for the given text.
    fn embed(&self, text: &str) -> Result<Vec<f32>, String>;

    /// Generate embeddings for multiple texts (batch operation).
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, String> {
        texts.iter().map(|t| self.embed(t)).collect()
    }
}

/// Trait for image embedding providers.
pub trait ImageEmbedder: Send + Sync {
    /// Generate an embedding vector for the given image data.
    fn embed(&self, image_data: &[u8]) -> Result<Vec<f32>, String>;
}

/// Stub text embedder for testing (returns random vectors).
pub struct StubTextEmbedder {
    dimension: usize,
}

impl StubTextEmbedder {
    /// Create a new stub embedder with the given dimension.
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

impl Default for StubTextEmbedder {
    fn default() -> Self {
        Self::new(384)
    }
}

impl TextEmbedder for StubTextEmbedder {
    fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        // Simple deterministic "embedding" based on text hash
        let hash = text.bytes().fold(0u64, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(b as u64)
        });

        let mut vec = Vec::with_capacity(self.dimension);
        let mut seed = hash;
        
        for _ in 0..self.dimension {
            // Simple LCG for deterministic pseudo-random numbers
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let val = ((seed / 65536) % 32768) as f32 / 32768.0;
            vec.push(val);
        }

        Ok(vec)
    }
}
