//! BM25 scoring implementation.

use std::collections::HashMap;

/// BM25 scorer for ranking documents.
#[derive(Debug, Clone)]
pub struct BM25Scorer {
    /// BM25 k1 parameter (term frequency saturation).
    pub k1: f32,
    /// BM25 b parameter (length normalization).
    pub b: f32,
}

impl Default for BM25Scorer {
    fn default() -> Self {
        Self {
            k1: 1.5,
            b: 0.75,
        }
    }
}

impl BM25Scorer {
    /// Create a new BM25 scorer with default parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate BM25 score for a document.
    ///
    /// # Arguments
    /// * `query_terms` - Terms from the query
    /// * `doc_terms` - Term frequencies in the document
    /// * `doc_length` - Total number of terms in the document
    /// * `avg_doc_length` - Average document length in the corpus
    /// * `doc_freq` - Document frequency for each term (how many docs contain it)
    /// * `total_docs` - Total number of documents in the corpus
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

            let norm_tf = (tf * (self.k1 + 1.0))
                / (tf + self.k1 * (1.0 - self.b + self.b * (doc_length as f32 / avg_doc_length)));

            score += idf * norm_tf;
        }

        score
    }

    /// Calculate inverse document frequency.
    fn idf(&self, doc_freq: f32, total_docs: usize) -> f32 {
        let n = total_docs as f32;
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
        
        let score = scorer.score(
            &query_terms,
            &doc_terms,
            6,
            10.0,
            &doc_freq,
            10,
        );
        
        assert!(score > 0.0);
    }
}
