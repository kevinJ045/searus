//! Text tokenization utilities for searchers.
//!
//! This module provides functions for breaking down text into tokens (words)
//! and for calculating term frequencies, which are essential steps for many
//! text-based search algorithms.

use unicode_segmentation::UnicodeSegmentation;

/// Tokenizes a given text into a vector of words.
///
/// This function performs the following steps:
/// 1. It uses Unicode segmentation to correctly split the text into words,
///    which is more robust than splitting by whitespace, especially for
///    multilingual text.
/// 2. It converts each word to lowercase to ensure that the tokenization is
///    case-insensitive (e.g., "Hello" and "hello" are treated as the same token).
///
/// # Arguments
///
/// * `text` - The string slice to be tokenized.
///
/// # Returns
///
/// A `Vec<String>` containing the lowercase word tokens.
pub fn tokenize(text: &str) -> Vec<String> {
  text
    .unicode_words()
    .map(|word| word.to_lowercase())
    .collect()
}

/// Calculates the frequency of each term in a given text.
///
/// This function first tokenizes the text using the `tokenize` function and
/// then counts the occurrences of each unique token.
///
/// # Arguments
///
/// * `text` - The string slice to be analyzed.
///
/// # Returns
///
/// A `HashMap<String, usize>` where the keys are the unique tokens (terms)
/// and the values are their frequencies (counts) in the text.
pub fn term_frequencies(text: &str) -> std::collections::HashMap<String, usize> {
  let tokens = tokenize(text);
  let mut freqs = std::collections::HashMap::new();

  for token in tokens {
    *freqs.entry(token).or_insert(0) += 1;
  }

  freqs
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_tokenize() {
    let text = "Hello, World! This is a test.";
    let tokens = tokenize(text);
    assert_eq!(tokens, vec!["hello", "world", "this", "is", "a", "test"]);
  }

  #[test]
  fn test_term_frequencies() {
    let text = "the quick brown fox jumps over the lazy dog";
    let freqs = term_frequencies(text);
    assert_eq!(freqs.get("the"), Some(&2));
    assert_eq!(freqs.get("quick"), Some(&1));
    assert_eq!(freqs.get("brown"), Some(&1));
  }
}
