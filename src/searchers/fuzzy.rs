//! Fuzzy search implementation using edit distance.

use crate::prelude::*;
use crate::searchers::tokenizer::tokenize;
use serde_json::Value;
use strsim::jaro_winkler;

/// Fuzzy searcher using string similarity.
pub struct FuzzySearch {
    /// Minimum similarity threshold (0.0 to 1.0).
    threshold: f64,
    /// Fields to search.
    fields: Vec<String>,
}

impl FuzzySearch {
    /// Create a new fuzzy searcher with default threshold (0.8).
    pub fn new(fields: Vec<String>) -> Self {
        Self {
            threshold: 0.8,
            fields,
        }
    }

    /// Set the similarity threshold.
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    /// Extract field value from an item.
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

impl<T> Searcher<T> for FuzzySearch
where
    T: serde::Serialize + Clone,
{
    fn kind(&self) -> SearcherKind {
        SearcherKind::Fuzzy
    }

    fn search(&self, query: &Query, items: &[T]) -> Vec<SearusMatch<T>> {
        let query_text = match &query.text {
            Some(text) => text,
            None => return Vec::new(),
        };

        let query_terms = tokenize(query_text);
        if query_terms.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();

        for item in items {
            let mut max_similarity = 0.0;
            let mut best_match = None;

            // Check each configured field
            for field_name in &self.fields {
                if let Some(text) = Self::extract_field(item, field_name) {
                    let doc_terms = tokenize(&text);

                    // Find best fuzzy match for each query term
                    for query_term in &query_terms {
                        for doc_term in &doc_terms {
                            let similarity = jaro_winkler(query_term, doc_term);
                            
                            if similarity > max_similarity && similarity >= self.threshold {
                                max_similarity = similarity;
                                best_match = Some((query_term.clone(), doc_term.clone()));
                            }
                        }
                    }
                }
            }

            if let Some((original, matched)) = best_match {
                let mut m = SearusMatch::new(item.clone(), max_similarity as f32);
                m.details.push(SearchDetail::Fuzzy {
                    matched_term: matched,
                    original_term: original,
                    similarity: max_similarity as f32,
                });

                results.push(m);
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        results
    }
}
