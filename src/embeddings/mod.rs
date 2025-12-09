//! Provides abstractions for generating embeddings from text and images.
//!
//! This module defines the core traits, `TextEmbedder` and `ImageEmbedder`,
//! which create a common interface for different embedding models. Embeddings
//! are vector representations of data that capture semantic meaning, and they
//! are the foundation of vector-based search.

/// A trait for providers that can generate embeddings from text.
///
/// The `Send` and `Sync` bounds are required to allow the embedder to be used
/// in a concurrent environment.
pub trait TextEmbedder: Send + Sync {
  /// Generates an embedding vector for a given string slice.
  ///
  /// # Arguments
  ///
  /// * `text` - The text to be embedded.
  ///
  /// # Returns
  ///
  /// A `Result` containing the embedding as a `Vec<f32>` on success, or an
  /// error string on failure.
  fn embed(&self, text: &str) -> Result<Vec<f32>, String>;

  /// Generates embeddings for a batch of string slices.
  ///
  /// This method provides a default implementation that iterates through the
  /// texts and calls `embed` for each one. Implementors can override this
  /// to provide a more efficient, batch-oriented implementation if their
  /// underlying model supports it.
  ///
  /// # Arguments
  ///
  /// * `texts` - A slice of texts to be embedded.
  ///
  /// # Returns
  ///
  /// A `Result` containing a vector of embeddings on success, or an error
  /// string on failure.
  fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, String> {
    texts.iter().map(|t| self.embed(t)).collect()
  }
}

/// A trait for providers that can generate embeddings from image data.
pub trait ImageEmbedder: Send + Sync {
  /// Generates an embedding vector for a given image.
  ///
  /// # Arguments
  ///
  /// * `image_data` - A byte slice representing the raw image data.
  ///
  /// # Returns
  ///
  /// A `Result` containing the embedding as a `Vec<f32>` on success, or an
  /// error string on failure.
  fn embed(&self, image_data: &[u8]) -> Result<Vec<f32>, String>;
}

/// A stub implementation of `TextEmbedder` for testing and demonstration.
///
/// This embedder does not use a real AI model. Instead, it generates
/// deterministic, pseudo-random vectors based on a hash of the input text.
/// This is useful for testing search functionality without needing to load a
/// large model.
pub struct StubTextEmbedder {
  /// The dimensionality of the vectors to be generated.
  dimension: usize,
}

impl StubTextEmbedder {
  /// Creates a new `StubTextEmbedder` with a specified vector dimension.
  pub fn new(dimension: usize) -> Self {
    Self { dimension }
  }
}

impl Default for StubTextEmbedder {
  /// Creates a `StubTextEmbedder` with a default dimension of 384.
  fn default() -> Self {
    Self::new(384)
  }
}

impl TextEmbedder for StubTextEmbedder {
  /// Generates a deterministic, pseudo-random embedding for the given text.
  ///
  /// The generation process uses a simple linear congruential generator (LCG)
  /// seeded with a hash of the input text to ensure that the same text always
  /// produces the same vector.
  fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
    // Create a simple hash from the text to seed the random number generator.
    let hash = text
      .bytes()
      .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));

    let mut vec = Vec::with_capacity(self.dimension);
    let mut seed = hash;

    for _ in 0..self.dimension {
      // Use a simple LCG for deterministic "random" numbers.
      seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
      let val = ((seed / 65536) % 32768) as f32 / 32768.0;
      vec.push(val);
    }

    Ok(vec)
  }
}
