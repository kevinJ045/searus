//! The main search engine that coordinates multiple searchers.

use crate::searcher::Searcher;
use crate::types::{Query, SearcherKind, SearusMatch};
use std::collections::HashMap;

/// The main search engine that coordinates multiple searchers.
pub struct SearusEngine<T> {
    searchers: Vec<Box<dyn Searcher<T>>>,
    normalization: NormalizationMethod,
}

impl<T: Default> SearusEngine<T> {
    /// Create a new engine builder.
    pub fn builder() -> SearusEngineBuilder<T> {
        SearusEngineBuilder::default()
    }

    /// Search using all registered searchers and merge results.
    pub fn search(&self, items: &[T], query: &Query) -> Vec<SearusMatch<T>>
    where
        T: Clone,
    {
        if self.searchers.is_empty() {
            return Vec::new();
        }

        // Collect results from all searchers
        let mut all_results: Vec<(SearcherKind, Vec<SearusMatch<T>>)> = Vec::new();

        for searcher in &self.searchers {
            let results = searcher.search(query, items);
            if !results.is_empty() {
                all_results.push((searcher.kind(), results));
            }
        }

        if all_results.is_empty() {
            return Vec::new();
        }

        // Normalize scores for each searcher's results
        let normalized_results = self.normalize_results(all_results);

        // Merge and rank results
        let merged = self.merge_results(normalized_results, query);

        // Apply pagination
        let skip = query.options.skip;
        let limit = query.options.limit;

        merged.into_iter().skip(skip).take(limit).collect()
    }

    /// Normalize scores from each searcher.
    fn normalize_results(
        &self,
        results: Vec<(SearcherKind, Vec<SearusMatch<T>>)>,
    ) -> Vec<(SearcherKind, Vec<SearusMatch<T>>)>
    where
        T: Clone,
    {
        results
            .into_iter()
            .map(|(kind, mut matches)| {
                if matches.is_empty() {
                    return (kind, matches);
                }

                // Find min and max scores
                let scores: Vec<f32> = matches.iter().map(|m| m.score).collect();
                let min_score = scores.iter().copied().fold(f32::INFINITY, f32::min);
                let max_score = scores.iter().copied().fold(f32::NEG_INFINITY, f32::max);

                // Normalize based on method
                match self.normalization {
                    NormalizationMethod::MinMax => {
                        let range = max_score - min_score;
                        if range > 0.0 {
                            for m in &mut matches {
                                m.score = (m.score - min_score) / range;
                            }
                        } else {
                            // All scores are the same
                            for m in &mut matches {
                                m.score = 1.0;
                            }
                        }
                    }
                    NormalizationMethod::InverseDistance => {
                        // Assume scores are distances, convert to similarities
                        for m in &mut matches {
                            m.score = 1.0 / (1.0 + m.score);
                        }
                    }
                }

                (kind, matches)
            })
            .collect()
    }

    /// Merge results from multiple searchers using weighted scoring.
    fn merge_results(
        &self,
        results: Vec<(SearcherKind, Vec<SearusMatch<T>>)>,
        query: &Query,
    ) -> Vec<SearusMatch<T>>
    where
        T: Clone,
    {
        // Group matches by item (using index as identifier)
        // This is a simplified version - in production we'd use proper entity IDs
        let mut merged: HashMap<usize, SearusMatch<T>> = HashMap::new();
        // let mut item_to_index: HashMap<usize, usize> = HashMap::new();

        for (kind, matches) in results {
            let weight = query.options.weights.get(&kind).copied().unwrap_or(1.0);

            for m in matches {
                // Find or create entry for this item
                // In a real implementation, we'd use proper entity IDs
                let item_hash = self.hash_item(&m.item);

                let entry = merged.entry(item_hash).or_insert_with(|| SearusMatch {
                    item: m.item.clone(),
                    score: 0.0,
                    field_scores: HashMap::new(),
                    details: Vec::new(),
                });

                // Add weighted score
                entry.score += m.score * weight;

                // Merge field scores
                for (field, score) in m.field_scores {
                    *entry.field_scores.entry(field).or_insert(0.0) += score * weight;
                }

                // Merge details
                entry.details.extend(m.details);
            }
        }

        // Convert to vec and sort by score (descending)
        let mut results: Vec<SearusMatch<T>> = merged.into_values().collect();
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// Simple hash function for items (placeholder - in production use proper IDs).
    fn hash_item(&self, _item: &T) -> usize {
        // This is a placeholder. In production, items should have proper IDs.
        // For now, we'll use memory address as a simple identifier.
        // This works for the basic case but isn't ideal.
        0
    }
}

/// Builder for the search engine.
#[derive(Default)]
pub struct SearusEngineBuilder<T> {
    searchers: Vec<Box<dyn Searcher<T>>>,
    normalization: Option<NormalizationMethod>,
}

impl<T> SearusEngineBuilder<T> {
    /// Add a searcher to the engine.
    pub fn with(mut self, searcher: Box<dyn Searcher<T>>) -> Self {
        self.searchers.push(searcher);
        self
    }

    /// Set the normalization method.
    pub fn normalization(mut self, method: NormalizationMethod) -> Self {
        self.normalization = Some(method);
        self
    }

    /// Build the engine.
    pub fn build(self) -> SearusEngine<T> {
        SearusEngine {
            searchers: self.searchers,
            normalization: self.normalization.unwrap_or(NormalizationMethod::MinMax),
        }
    }
}

/// Score normalization methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizationMethod {
    /// Min-Max normalization: (score - min) / (max - min)
    MinMax,
    /// Inverse distance: 1 / (1 + distance)
    InverseDistance,
}
