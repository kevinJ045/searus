//! The main search engine that coordinates multiple searchers.

use crate::searcher::Searcher;
use crate::types::{Query, SearcherKind, SearusMatch};
use std::collections::HashMap;

/// The main search engine that coordinates multiple searchers.
///
/// `SearusEngine` is the central component of the library. It manages a collection
/// of `Searcher` plugins, dispatches search queries to them, and then merges
/// the results into a single, ranked list.
pub struct SearusEngine<T> {
  /// The collection of registered searcher plugins.
  searchers: Vec<Box<dyn Searcher<T>>>,
  /// The method used to normalize scores from different searchers.
  normalization: NormalizationMethod,
}

impl<T> SearusEngine<T> {
  /// Creates a new `SearusEngineBuilder` to construct an engine.
  pub fn builder() -> SearusEngineBuilder<T> {
    SearusEngineBuilder::new()
  }

  /// Searches for items using all registered searchers and merges the results.
  ///
  /// The search process involves the following steps:
  /// 1. The query is dispatched to each registered `Searcher`.
  /// 2. The raw scores from each searcher are normalized to a common scale
  ///    (e.g., 0.0 to 1.0) using the configured `NormalizationMethod`.
  /// 3. The normalized results are merged. If multiple searchers match the same
  ///    item, their scores are combined using a weighted sum.
  /// 4. The final, merged list is sorted by score in descending order.
  /// 5. Pagination (`skip` and `limit`) is applied to the final list.
  ///
  /// # Arguments
  ///
  /// * `items` - A slice of items to be searched.
  /// * `query` - The search query containing the search parameters.
  ///
  /// # Returns
  ///
  /// A `Vec<SearusMatch<T>>` containing the final, ranked search results.
  pub fn search(&self, items: &[T], query: &Query) -> Vec<SearusMatch<T>>
  where
    T: Clone,
  {
    if self.searchers.is_empty() {
      return Vec::new();
    }

    // Collect results from all searchers
    let all_results: Vec<(SearcherKind, Vec<SearusMatch<T>>)> = self
      .searchers
      .iter()
      .map(|searcher| (searcher.kind(), searcher.search(query, items)))
      .filter(|(_, results)| !results.is_empty())
      .collect();

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

  /// Normalizes the scores from each searcher to a common scale.
  ///
  /// This is a private helper method that ensures that scores from different
  /// searchers (which may have vastly different scales) can be meaningfully
  /// combined.
  fn normalize_results(
    &self,
    results: Vec<(SearcherKind, Vec<SearusMatch<T>>)>,
  ) -> Vec<(SearcherKind, Vec<SearusMatch<T>>)> {
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
              // All scores are the same, so we can set them all to 1.0
              for m in &mut matches {
                m.score = 1.0;
              }
            }
          }
          NormalizationMethod::InverseDistance => {
            // Assumes scores are distances; converts them to similarities.
            for m in &mut matches {
              m.score = 1.0 / (1.0 + m.score);
            }
          }
        }

        (kind, matches)
      })
      .collect()
  }

  /// Merges results from multiple searchers using a weighted scoring model.
  ///
  /// This method groups matches by item and combines their scores.
  ///
  /// # Warning on Merging Logic
  ///
  /// The current implementation uses a placeholder `hash_item` function to
  /// identify unique items. This function is not robust and may not correctly
  /// merge results for the same item if the item type `T` does not have a
  /// stable identity (e.g., if it's cloned). For production use, it is
  /// highly recommended that the items being searched have a proper, unique
  /// identifier.
  fn merge_results(
    &self,
    results: Vec<(SearcherKind, Vec<SearusMatch<T>>)>,
    query: &Query,
  ) -> Vec<SearusMatch<T>>
  where
    T: Clone,
  {
    let mut merged: HashMap<usize, SearusMatch<T>> = HashMap::new();

    for (kind, matches) in results {
      let weight = query.options.weights.get(&kind).copied().unwrap_or(1.0);

      for m in matches {
        // Use a placeholder hash to identify the item.
        let item_hash = m.id;

        let entry = merged.entry(item_hash).or_insert_with(|| SearusMatch {
          id: 0 as usize,
          item: m.item.clone(),
          score: 0.0,
          field_scores: HashMap::new(),
          details: Vec::new(),
        });

        // Add the weighted score to the total.
        entry.score += m.score * weight;

        // Merge field scores.
        for (field, score) in m.field_scores {
          *entry.field_scores.entry(field).or_insert(0.0) += score * weight;
        }

        // Merge details from all searchers.
        entry.details.extend(m.details);
      }
    }

    // Convert the map of merged items to a Vec and sort by score.
    let mut results: Vec<SearusMatch<T>> = merged.into_values().collect();
    results.sort_by(|a, b| {
      b.score
        .partial_cmp(&a.score)
        .unwrap_or(std::cmp::Ordering::Equal)
    });

    results
  }
}

/// A builder for creating `SearusEngine` instances.
///
/// The builder pattern provides a convenient way to configure and construct
/// a `SearusEngine` with the desired searchers and normalization method.
#[derive(Default)]
pub struct SearusEngineBuilder<T> {
  searchers: Vec<Box<dyn Searcher<T>>>,
  normalization: Option<NormalizationMethod>,
}

impl<T> SearusEngineBuilder<T> {
  /// Creates a new, empty `SearusEngineBuilder`.
  pub fn new() -> Self {
    Self {
      searchers: Vec::new(),
      normalization: None,
    }
  }

  /// Adds a searcher plugin to the engine.
  ///
  /// Searchers are added as boxed traits to allow for different underlying
  /// implementations.
  pub fn with(mut self, searcher: Box<dyn Searcher<T>>) -> Self {
    // pub fn with(mut self, searcher: impl Searcher<T> + 'static) -> Self {
    self.searchers.push(searcher);
    self
  }

  /// Sets the score normalization method for the engine.
  ///
  /// If not set, `NormalizationMethod::MinMax` is used by default.
  pub fn normalization(mut self, method: NormalizationMethod) -> Self {
    self.normalization = Some(method);
    self
  }

  /// Builds the `SearusEngine` with the configured components.
  pub fn build(self) -> SearusEngine<T> {
    SearusEngine {
      searchers: self.searchers,
      normalization: self.normalization.unwrap_or(NormalizationMethod::MinMax),
    }
  }
}

/// Defines the methods for normalizing scores from different searchers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizationMethod {
  /// Min-Max normalization, which scales scores to a [0, 1] range.
  /// The formula is: `(score - min) / (max - min)`.
  MinMax,
  /// Inverse distance normalization, used when scores represent distances
  /// (where lower is better). It converts distances to similarities.
  /// The formula is: `1 / (1 + distance)`.
  InverseDistance,
}
