//! An implementation of the Okapi BM25 scoring algorithm.
//!
//! BM25 (Best Matching 25) is a ranking function used by search engines to
//! estimate the relevance of documents to a given search query.

use std::collections::HashMap;

/// A scorer for ranking documents using the BM25 algorithm.
///
/// This struct holds the configuration parameters for BM25 and provides the
/// method to calculate the score.
#[derive(Debug, Clone)]
pub struct BM25Scorer {
  /// The `k1` parameter controls the term frequency saturation. A higher value
  /// means that the score continues to increase with term frequency, while a
  /// lower value means the score saturates more quickly. The default is 1.5.
  pub k1: f32,
  /// The `b` parameter controls the document length normalization. A value of
  /// 0.0 means no length normalization, while a value of 1.0 means full
  /// normalization. The default is 0.75.
  pub b: f32,
}

impl Default for BM25Scorer {
  /// Creates a `BM25Scorer` with the default `k1` and `b` parameters.
  fn default() -> Self {
    Self { k1: 1.5, b: 0.75 }
  }
}

impl BM25Scorer {
  /// Creates a new `BM25Scorer` with the default parameters.
  pub fn new() -> Self {
    Self::default()
  }

  /// Calculates the BM25 score of a document for a given query.
  ///
  /// The BM25 score is a sum of the scores for each query term. The score for
  /// each term is a product of its Inverse Document Frequency (IDF) and a
  /// normalized term frequency (TF).
  ///
  /// # Arguments
  ///
  /// * `query_terms` - A slice of the terms in the search query.
  /// * `doc_terms` - A map of term frequencies for the document being scored.
  /// * `doc_length` - The total number of terms in the document.
  /// * `avg_doc_length` - The average document length across the entire corpus.
  /// * `doc_freq` - A map of document frequencies for each term in the corpus.
  /// * `total_docs` - The total number of documents in the corpus.
  ///
  /// # Returns
  ///
  /// The calculated BM25 score as an `f32`.
  pub fn score(
    &self,
    query_terms: &[String],
    doc_terms: &HashMap<String, usize>,
    doc_length: usize,
    avg_doc_length: f32,
    doc_freq: &HashMap<String, usize>,
    total_docs: usize,
  ) -> f32 {
    let mut score = 0.0;

    for term in query_terms {
      let tf = *doc_terms.get(term).unwrap_or(&0) as f32;
      if tf == 0.0 {
        continue;
      }

      let df = *doc_freq.get(term).unwrap_or(&1) as f32;
      let idf = self.idf(df, total_docs);

      // Calculate the normalized term frequency component.
      let norm_tf = (tf * (self.k1 + 1.0))
        / (tf + self.k1 * (1.0 - self.b + self.b * (doc_length as f32 / avg_doc_length)));

      score += idf * norm_tf;
    }

    score
  }

  /// Calculates the Inverse Document Frequency (IDF) for a term.
  ///
  /// IDF is a measure of how much information a word provides, i.e., whether
  /// it's common or rare across all documents.
  fn idf(&self, doc_freq: f32, total_docs: usize) -> f32 {
    let n = total_docs as f32;
    // This is a common variant of the IDF formula.
    ((n - doc_freq + 0.5) / (doc_freq + 0.5) + 1.0).ln()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_bm25_scoring() {
    let scorer = BM25Scorer::new();

    let query_terms = vec!["rust".to_string(), "search".to_string()];

    let mut doc_terms = HashMap::new();
    doc_terms.insert("rust".to_string(), 3);
    doc_terms.insert("search".to_string(), 2);
    doc_terms.insert("engine".to_string(), 1);

    let mut doc_freq = HashMap::new();
    doc_freq.insert("rust".to_string(), 5);
    doc_freq.insert("search".to_string(), 3);

    let score = scorer.score(&query_terms, &doc_terms, 6, 10.0, &doc_freq, 10);

    assert!(score > 0.0);
  }
}
