//! A `Searcher` implementation for fuzzy (approximate) string matching.

use crate::context::SearchContext;
use crate::prelude::*;
use crate::searchers::tokenizer::tokenize;
use serde_json::Value;
use strsim::jaro_winkler;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

#[cfg(feature = "parallel")]
pub trait FuzzySearchable: serde::Serialize + Clone + Send + Sync {}
#[cfg(feature = "parallel")]
impl<T: serde::Serialize + Clone + Send + Sync> FuzzySearchable for T {}

#[cfg(not(feature = "parallel"))]
pub trait FuzzySearchable: serde::Serialize + Clone {}
#[cfg(not(feature = "parallel"))]
impl<T: serde::Serialize + Clone> FuzzySearchable for T {}

/// A searcher that performs fuzzy string matching using the Jaro-Winkler similarity algorithm.
///
/// `FuzzySearch` is useful for finding matches that are not exact, which can help
/// with typos or variations in spelling. It works by tokenizing the query and
/// the text in the specified fields, and then comparing the tokens to find
/// terms with a high degree of similarity.
pub struct FuzzySearch {
  /// The minimum similarity threshold required to consider a term a match.
  /// This value should be between 0.0 (no similarity) and 1.0 (exact match).
  threshold: f64,
  /// The names of the fields to search within the items. The items are expected
  /// to be serializable to a JSON-like structure to allow for field extraction.
  fields: Vec<String>,
}

impl FuzzySearch {
  /// Creates a new `FuzzySearch` instance with a default threshold of 0.8.
  ///
  /// # Arguments
  ///
  /// * `fields` - A `Vec<String>` containing the names of the fields to be
  ///   searched.
  pub fn new(fields: Vec<String>) -> Self {
    Self {
      threshold: 0.8,
      fields,
    }
  }

  /// Sets a custom similarity threshold for the fuzzy searcher.
  ///
  /// # Arguments
  ///
  /// * `threshold` - The desired threshold, from 0.0 to 1.0.
  pub fn with_threshold(mut self, threshold: f64) -> Self {
    self.threshold = threshold;
    self
  }

  /// Extracts the value of a specified field from a serializable item.
  ///
  /// This helper function serializes the item to a `serde_json::Value` and then
  /// extracts the text from the specified field. It can handle string and
  /// number fields (by converting numbers to strings).
  fn extract_field<T>(item: &T, field: &str) -> Option<String>
  where
    T: serde::Serialize,
  {
    let value = serde_json::to_value(item).ok()?;
    let field_value = value.get(field)?;

    match field_value {
      Value::String(s) => Some(s.clone()),
      Value::Number(n) => Some(n.to_string()),
      _ => None,
    }
  }
}

impl FuzzySearch {
  /// Match a single entity against the query.
  pub fn match_entity<T>(
    &self,
    item: &T,
    index: usize,
    _query: &Query,
    query_terms: &[String],
  ) -> Option<SearusMatch<T>>
  where
    T: FuzzySearchable,
  {
    let mut max_similarity = 0.0;
    let mut best_query_term = String::new();
    let mut best_doc_term = String::new();

    // Check each configured field for a fuzzy match.
    'outer: for field_name in &self.fields {
      if let Some(text) = Self::extract_field(item, field_name) {
        let doc_terms = tokenize(&text);

        // Find the best fuzzy match between query terms and document terms.
        for query_term in query_terms {
          let query_len = query_term.len();
          
          for doc_term in &doc_terms {
            // OPTIMIZATION: Length-based pruning
            // Skip if length difference is too large (>50% different)
            let doc_len = doc_term.len();
            let len_diff = if query_len > doc_len {
              query_len - doc_len
            } else {
              doc_len - query_len
            };
            let max_len = query_len.max(doc_len);
            if max_len > 0 && (len_diff * 2) > max_len {
              continue;
            }

            let similarity = jaro_winkler(query_term, doc_term);

            if similarity > max_similarity && similarity >= self.threshold {
              max_similarity = similarity;
              best_query_term = query_term.clone();
              best_doc_term = doc_term.clone();
              
              // OPTIMIZATION: Early cutoff if we find a near-perfect match
              if similarity > 0.95 {
                break 'outer;
              }
            }
          }
        }
      }
    }

    // If a match was found above the threshold, create a SearusMatch.
    if max_similarity >= self.threshold {
      let mut m = SearusMatch::new(item.clone(), max_similarity as f32, index);
      m.details.push(SearchDetail::Fuzzy {
        matched_term: best_doc_term,
        original_term: best_query_term,
        similarity: max_similarity as f32,
      });

      Some(m)
    } else {
      None
    }
  }

  /// Sort the search results.
  #[cfg(feature = "parallel")]
  pub fn sort_results<T: Send + Sync>(&self, results: &mut Vec<SearusMatch<T>>) {
    results.par_sort_by(|a, b| {
      b.score
        .partial_cmp(&a.score)
        .unwrap_or(std::cmp::Ordering::Equal)
    });
  }

  /// Sort the search results.
  #[cfg(not(feature = "parallel"))]
  pub fn sort_results<T>(&self, results: &mut Vec<SearusMatch<T>>) {
    results.sort_by(|a, b| {
      b.score
        .partial_cmp(&a.score)
        .unwrap_or(std::cmp::Ordering::Equal)
    });
  }
}

impl<T> Searcher<T> for FuzzySearch
where
  T: FuzzySearchable,
{
  fn kind(&self) -> SearcherKind {
    SearcherKind::Fuzzy
  }

  /// Performs a fuzzy search for the query text within the specified fields of the items.
  ///
  /// This method tokenizes the query text and the text from each of the configured
  /// fields in the items. It then uses the Jaro-Winkler algorithm to find the
  /// similarity between each query term and each document term.
  ///
  /// If a pair of terms has a similarity score that exceeds the configured
  /// threshold, it is considered a match. The highest similarity score found
  /// for an item is used as its raw score.
  fn search(&self, context: &SearchContext<T>, query: &Query) -> Vec<SearusMatch<T>> {
    let items = context.items;
    let query_text = match &query.text {
      Some(text) => text,
      None => return Vec::new(),
    };

    let query_terms = tokenize(query_text);
    if query_terms.is_empty() {
      return Vec::new();
    }

    #[cfg(feature = "parallel")]
    let mut results: Vec<SearusMatch<T>> = {
      // OPTIMIZATION: Pre-allocate result vector
      let matches: Vec<_> = items
        .par_iter()
        .enumerate()
        .filter(|(_, item)| {
           if let Some(filters) = &query.filters {
             filters.evaluate(item)
           } else {
             true
           }
        })
        .filter_map(|(index, item)| self.match_entity(item, index, query, &query_terms))
        .collect();
      
      let mut results = Vec::with_capacity(matches.len());
      results.extend(matches);
      results
    };

    #[cfg(not(feature = "parallel"))]
    let mut results: Vec<SearusMatch<T>> = {
      // OPTIMIZATION: Pre-allocate with estimated capacity
      let mut results = Vec::with_capacity(items.len() / 20); // Fuzzy matches are typically rare
      results.extend(
        items
          .iter()
          .enumerate()
          .filter(|(_, item)| {
             if let Some(filters) = &query.filters {
               filters.evaluate(item)
             } else {
               true
             }
          })
          .filter_map(|(index, item)| self.match_entity(item, index, query, &query_terms))
      );
      results
    };

    // Sort results by score in descending order.
    self.sort_results(&mut results);

    results
  }
}
