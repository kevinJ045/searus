//! Semantic text search implementation.

use crate::prelude::*;
use crate::searchers::bm25::BM25Scorer;
use crate::searchers::tokenizer::{term_frequencies, tokenize};
use serde_json::Value;
use std::collections::HashMap;
#[cfg(not(feature = "parallel"))]
use std::collections::HashSet;
use std::fmt::Debug;

#[cfg(feature = "parallel")]
use dashmap::DashMap;
#[cfg(feature = "parallel")]
use rayon::prelude::*;
#[cfg(feature = "parallel")]
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(feature = "parallel")]
pub trait SemanticSearchable: serde::Serialize + Clone + Debug + Send + Sync {}
#[cfg(feature = "parallel")]
impl<T: serde::Serialize + Clone + Debug + Send + Sync> SemanticSearchable for T {}

#[cfg(not(feature = "parallel"))]
pub trait SemanticSearchable: serde::Serialize + Clone + Debug {}
#[cfg(not(feature = "parallel"))]
impl<T: serde::Serialize + Clone + Debug> SemanticSearchable for T {}

/// Semantic text searcher using BM25 and tokenization.
pub struct SemanticSearch {
  rules: SemanticRules,
  bm25: BM25Scorer,
}

impl SemanticSearch {
  /// Create a new semantic searcher with the given rules.
  pub fn new(rules: SemanticRules) -> Self {
    Self {
      rules,
      bm25: BM25Scorer::new(),
    }
  }

  /// Extract field value from an item using serde_json.
  fn extract_field<T>(item: &T, field: &str) -> Option<String>
  where
    T: serde::Serialize,
  {
    // Serialize to JSON value for field access
    let value = serde_json::to_value(item).ok()?;
    Self::get_nested_field(&value, field)
  }

  /// Get a nested field from a JSON value.
  fn get_nested_field(value: &Value, path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;

    for part in parts {
      current = current.get(part)?;
    }

    match current {
      Value::String(s) => Some(s.clone()),
      Value::Number(n) => Some(n.to_string()),
      Value::Bool(b) => Some(b.to_string()),
      _ => None,
    }
  }

  /// Calculate corpus statistics for BM25.
  fn calculate_corpus_stats<T>(items: &[T], rules: &SemanticRules) -> CorpusStats
  where
    T: serde::Serialize + Searchable,
  {
    // --- Parallel version ---
    #[cfg(feature = "parallel")]
    {
      // Each thread computes partial stats
      let doc_freq: DashMap<String, AtomicUsize> = DashMap::new();
      let total_length = AtomicUsize::new(0);
      let doc_count = AtomicUsize::new(0);

      items.par_iter().for_each(|item| {
        let mut terms = std::collections::HashSet::new();

        for (field_name, _) in &rules.fields {
          if let Some(text) = Self::extract_field(item, field_name) {
            let tokens = tokenize(&text);

            total_length.fetch_add(tokens.len(), Ordering::Relaxed);
            doc_count.fetch_add(1, Ordering::Relaxed);

            for t in tokens {
              terms.insert(t);
            }
          }
        }

        for t in terms {
          doc_freq
            .entry(t)
            .or_insert_with(|| AtomicUsize::new(0))
            .fetch_add(1, Ordering::Relaxed);
        }
      });

      let df_map: HashMap<String, usize> = doc_freq
        .into_iter()
        .map(|(k, v)| (k, v.load(Ordering::Relaxed)))
        .collect();

      let total_len = total_length.load(Ordering::Relaxed);
      let docs = doc_count.load(Ordering::Relaxed);

      return CorpusStats {
        doc_freq: df_map,
        avg_doc_length: (total_len as f32) / (docs as f32),
        total_docs: items.len(),
      };
    }

    // --- Sequential version ---
    #[cfg(not(feature = "parallel"))]
    {
      let mut doc_freq: HashMap<String, usize> = HashMap::new();
      let mut total_length = 0;
      let mut doc_count = 0;

      for item in items {
        let mut doc_terms = HashSet::new();

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
}

pub struct CorpusStats {
  doc_freq: HashMap<String, usize>,
  avg_doc_length: f32,
  total_docs: usize,
}

impl<T> Searcher<T> for SemanticSearch
where
  T: SemanticSearchable,
{
  fn kind(&self) -> SearcherKind {
    SearcherKind::Semantic
  }

  fn search(&self, query: &Query, items: &[T]) -> Vec<SearusMatch<T>> {
    // Only process if there's a text query
    let query_text = match &query.text {
      Some(text) => text,
      None => return Vec::new(),
    };

    if items.is_empty() {
      return Vec::new();
    }

    // Tokenize query
    let query_terms = tokenize(query_text);
    if query_terms.is_empty() {
      return Vec::new();
    }

    // Calculate corpus statistics
    let stats = Self::calculate_corpus_stats(items, &self.rules);

    // Score each item
    #[cfg(feature = "parallel")]
    let mut results: Vec<SearusMatch<T>> = {
      // OPTIMIZATION: Collect into pre-allocated vector
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
        .filter_map(|(index, item)| self.match_entity(item, index, query, &stats, &query_terms))
        .collect();

      let mut results = Vec::with_capacity(matches.len());
      results.extend(matches);
      results
    };

    #[cfg(not(feature = "parallel"))]
    let mut results: Vec<SearusMatch<T>> = {
      // OPTIMIZATION: Pre-allocate with estimated capacity
      let mut results = Vec::with_capacity(items.len() / 10); // Assume ~10% match rate
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
          .filter_map(|(index, item)| self.match_entity(item, index, query, &stats, &query_terms)),
      );
      results
    };

    // Sort by score descending
    self.sort_results(&mut results);

    results
  }
}

impl SemanticSearch {
  /// Score a single field.
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
        // Exact match (case-insensitive)
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
        // BM25 scoring
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

        // Track matched terms
        for term in query_terms {
          if doc_terms.contains_key(term) {
            matched_terms.push(term.clone());
          }
        }

        score
      }
      Matcher::Tokenized => {
        // Simple token matching with term frequency
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
        // Fuzzy matching handled by FuzzySearch
        0.0
      }
    }
  }
}

impl SemanticSearch {
  /// Match a single entity against the query.
  pub fn match_entity<T>(
    &self,
    item: &T,
    index: usize,
    _query: &Query,
    stats: &CorpusStats,
    query_terms: &[String],
  ) -> Option<SearusMatch<T>>
  where
    T: SemanticSearchable,
  {
    let mut total_score = 0.0;
    let mut field_scores = HashMap::new();
    let mut matched_terms = Vec::new();

    // Score each configured field
    for (field_name, field_rule) in &self.rules.fields {
      if let Some(text) = Self::extract_field(item, field_name) {
        let field_score =
          self.score_field(query_terms, &text, field_rule, stats, &mut matched_terms);

        if field_score > 0.0 {
          let weighted_score = field_score * field_rule.boost * field_rule.priority as f32;
          field_scores.insert(field_name.clone(), weighted_score);
          total_score += weighted_score;
        }
      }
    }

    if total_score > 0.0 {
      let mut m = SearusMatch::new(item.clone(), total_score, index);
      m.field_scores = field_scores;

      if !matched_terms.is_empty() {
        m.details.push(SearchDetail::Semantic {
          matched_terms: matched_terms.clone(),
          field: "multiple".to_string(),
          weight: total_score,
        });
      }

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

  #[cfg(not(feature = "parallel"))]
  pub fn sort_results<T>(&self, results: &mut Vec<SearusMatch<T>>) {
    results.sort_by(|a, b| {
      b.score
        .partial_cmp(&a.score)
        .unwrap_or(std::cmp::Ordering::Equal)
    });
  }
}
