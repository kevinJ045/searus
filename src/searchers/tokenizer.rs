//! Text tokenization utilities.

use unicode_segmentation::UnicodeSegmentation;

/// Tokenize text into words.
pub fn tokenize(text: &str) -> Vec<String> {
  text
    .unicode_words()
    .map(|word| word.to_lowercase())
    .collect()
}

/// Calculate term frequencies for a text.
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
