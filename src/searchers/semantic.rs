//! Semantic text search implementation.

use crate::prelude::*;
use crate::searchers::tokenizer::{tokenize, term_frequencies};
use crate::searchers::bm25::BM25Scorer;
use serde_json::Value;
use std::collections::HashMap;

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
        T: serde::Serialize,
    {
        let mut doc_freq: HashMap<String, usize> = HashMap::new();
        let mut total_length = 0;
        let mut doc_count = 0;

        for item in items {
            let mut doc_terms = std::collections::HashSet::new();

            // Collect terms from all configured fields
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

            // Update document frequencies
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

struct CorpusStats {
    doc_freq: HashMap<String, usize>,
    avg_doc_length: f32,
    total_docs: usize,
}

impl<T> Searcher<T> for SemanticSearch
where
    T: serde::Serialize + Clone,
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
        let mut results = Vec::new();

        for item in items {
            let mut total_score = 0.0;
            let mut field_scores = HashMap::new();
            let mut matched_terms = Vec::new();

            // Score each configured field
            for (field_name, field_rule) in &self.rules.fields {
                if let Some(text) = Self::extract_field(item, field_name) {
                    let field_score = self.score_field(
                        &query_terms,
                        &text,
                        field_rule,
                        &stats,
                        &mut matched_terms,
                    );

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
                        matched_terms: matched_terms.clone(),
                        field: "multiple".to_string(),
                        weight: total_score,
                    });
                }

                results.push(m);
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

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
