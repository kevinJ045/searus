//! A `Searcher` implementation for semantic text search.
//!
//! This module provides `SemanticSearch`, a sophisticated text searcher that uses
//! the BM25 algorithm for relevance scoring and can be configured with a
//! detailed set of `SemanticRules`.

use crate::prelude::*;
use crate::searchers::bm25::BM25Scorer;
use crate::searchers::tokenizer::{term_frequencies, tokenize};
use serde_json::Value;
use std::collections::HashMap;

/// A searcher for semantic text analysis using BM25 scoring and configurable rules.
///
/// `SemanticSearch` allows for fine-grained control over how text fields are
/// searched and scored. It uses `SemanticRules` to define which fields to search,
/// what matching strategy to apply, and how to weight their importance.
pub struct SemanticSearch {
  /// The set of rules that configure the behavior of the searcher.
  rules: SemanticRules,
  /// The BM25 scorer instance used for relevance calculation.
  bm25: BM25Scorer,
}

impl SemanticSearch {
  /// Creates a new `SemanticSearch` instance with a given set of rules.
  pub fn new(rules: SemanticRules) -> Self {
    Self {
      rules,
      bm25: BM25Scorer::new(),
    }
  }

  /// Extracts the value of a field from a serializable item.
  ///
  /// This function can extract values from nested fields using dot notation
  /// (e.g., "author.name").
  fn extract_field<T>(item: &T, field: &str) -> Option<String>
  where
    T: serde::Serialize,
  {
    let value = serde_json::to_value(item).ok()?;
    Self::get_nested_field(&value, field)
  }

  /// Recursively gets a nested field from a `serde_json::Value`.
  fn get_nested_field(value: &Value, path: &str) -> Option<String> {
    let mut parts = path.split('.');
    let mut current = value;

    while let Some(part) = parts.next() {
      current = current.get(part)?;
    }

    match current {
      Value::String(s) => Some(s.clone()),
      Value::Number(n) => Some(n.to_string()),
      Value::Bool(b) => Some(b.to_string()),
      _ => None,
    }
  }

  /// Calculates corpus-wide statistics required for BM25 scoring.
  ///
  /// This function iterates through all items and all configured fields to
  /// compute the document frequency for each term, the average document length,
  /// and the total number of documents. These stats are crucial for the IDF
  /// (Inverse Document Frequency) part of the BM25 algorithm.
  fn calculate_corpus_stats<T>(items: &[T], rules: &SemanticRules) -> CorpusStats
  where
    T: serde::Serialize,
  {
    let mut doc_freq: HashMap<String, usize> = HashMap::new();
    let mut total_length = 0;
    let mut doc_count = 0;

    for item in items {
      let mut doc_terms = std::collections::HashSet::new();

      // Collect terms from all fields defined in the rules.
      for (field_name, _) in &rules.fields {
        if let Some(text) = Self::extract_field(item, field_name) {
          let tokens = tokenize(&text);
          total_length += tokens.len();
          doc_count += 1;

          for token in tokens {
            doc_terms.insert(token);
          }
        }
      }

      // Update the document frequency for each unique term in the document.
      for term in doc_terms {
        *doc_freq.entry(term).or_insert(0) += 1;
      }
    }

    let avg_doc_length = if doc_count > 0 {
      total_length as f32 / doc_count as f32
    } else {
      0.0
    };

    CorpusStats {
      doc_freq,
      avg_doc_length,
      total_docs: items.len(),
    }
  }
}

/// A container for corpus-wide statistics needed for BM25.
struct CorpusStats {
  /// A map from a term to the number of documents it appears in.
  doc_freq: HashMap<String, usize>,
  /// The average length of a document in the corpus.
  avg_doc_length: f32,
  /// The total number of documents in the corpus.
  total_docs: usize,
}

impl<T> Searcher<T> for SemanticSearch
where
  T: serde::Serialize + Clone,
{
  fn kind(&self) -> SearcherKind {
    SearcherKind::Semantic
  }

  /// Performs a semantic search using the configured rules and BM25 scoring.
  ///
  /// The search process is as follows:
  /// 1. Corpus statistics (like document frequencies) are calculated.
  /// 2. For each item, each field specified in the `SemanticRules` is scored.
  /// 3. The score for each field is calculated based on its configured `Matcher`.
  /// 4. The final score for an item is a weighted sum of its field scores,
  ///    taking into account the `boost` and `priority` from the rules.
  fn search(&self, query: &Query, items: &[T]) -> Vec<SearusMatch<T>> {
    let query_text = match &query.text {
      Some(text) => text,
      None => return Vec::new(),
    };

    if items.is_empty() {
      return Vec::new();
    }

    let query_terms = tokenize(query_text);
    if query_terms.is_empty() {
      return Vec::new();
    }

    // Pre-calculate statistics for the entire corpus.
    let stats = Self::calculate_corpus_stats(items, &self.rules);

    let mut results = Vec::new();

    for item in items {
      let mut total_score = 0.0;
      let mut field_scores = HashMap::new();
      let mut matched_terms = Vec::new();

      // Score each field according to the rules.
      for (field_name, field_rule) in &self.rules.fields {
        if let Some(text) = Self::extract_field(item, field_name) {
          let field_score =
            self.score_field(&query_terms, &text, field_rule, &stats, &mut matched_terms);

          if field_score > 0.0 {
            let weighted_score = field_score * field_rule.boost * field_rule.priority as f32;
            field_scores.insert(field_name.clone(), weighted_score);
            total_score += weighted_score;
          }
        }
      }

      if total_score > 0.0 {
        let mut m = SearusMatch::new(item.clone(), total_score);
        m.field_scores = field_scores;

        if !matched_terms.is_empty() {
          m.details.push(SearchDetail::Semantic {
            matched_terms: matched_terms.into_iter().collect(),
            field: "multiple".to_string(), // Field could be generalized
            weight: total_score,
          });
        }

        results.push(m);
      }
    }

    // Sort results by score in descending order.
    results.sort_by(|a, b| {
      b.score
        .partial_cmp(&a.score)
        .unwrap_or(std::cmp::Ordering::Equal)
    });

    results
  }
}

impl SemanticSearch {
  /// Scores a single field based on the provided rule and query.
  fn score_field(
    &self,
    query_terms: &[String],
    text: &str,
    rule: &FieldRule,
    stats: &CorpusStats,
    matched_terms: &mut Vec<String>,
  ) -> f32 {
    match rule.matcher {
      Matcher::Exact => {
        let text_lower = text.to_lowercase();
        let query_lower = query_terms.join(" ");
        if text_lower.contains(&query_lower) {
          matched_terms.extend(query_terms.iter().cloned());
          1.0
        } else {
          0.0
        }
      }
      Matcher::BM25 => {
        let doc_terms = term_frequencies(text);
        let doc_length = tokenize(text).len();

        let score = self.bm25.score(
          query_terms,
          &doc_terms,
          doc_length,
          stats.avg_doc_length,
          &stats.doc_freq,
          stats.total_docs,
        );

        for term in query_terms {
          if doc_terms.contains_key(term) {
            matched_terms.push(term.clone());
          }
        }
        score
      }
      Matcher::Tokenized => {
        let doc_terms = term_frequencies(text);
        let mut score = 0.0;

        for term in query_terms {
          if let Some(&freq) = doc_terms.get(term) {
            matched_terms.push(term.clone());
            score += freq as f32;
          }
        }
        score
      }
      Matcher::Fuzzy => {
        // Fuzzy matching is expected to be handled by the FuzzySearcher.
        0.0
      }
    }
  }
}
