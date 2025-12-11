//! The main search engine that coordinates multiple searchers.

use crate::context::SearchContext;
use crate::extension::SearusExtension;
use crate::searcher::Searcher;
use crate::types::{Query, Searchable, SearcherKind, SearusMatch};
use std::collections::HashMap;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

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
  /// The collection of registered extensions.
  extensions: Vec<Box<dyn SearusExtension<T>>>,
}

impl<T: Searchable> SearusEngine<T> {
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
    // Clone query to allow modification by extensions
    let mut query = query.clone();

    // Hook: before_query
    for ext in &self.extensions {
      ext.before_query(&mut query);
    }

    // Prepare items, potentially modified by extensions
    // We only clone if there are extensions that might modify items,
    // otherwise we use the slice directly (optimization needed later, for now simple path)
    // Actually, since before_items takes &mut Vec<T>, we must have a Vec.
    // If we want to support adding items, we need a Vec.
    let mut items_vec = if !self.extensions.is_empty() {
      items.to_vec()
    } else {
      // If no extensions, we might not need a vec, but the current logic below uses items slice.
      // However, if we want to support before_items, we need to handle the case where items change.
      // For simplicity and correctness with extensions, let's create the vec if extensions exist.
      // But wait, if we create a vec, we need to pass that vec to searchers.
      // Searchers take &[T].
      Vec::new()
    };

    // If we have extensions, populate items_vec and run hooks
    let items_slice = if !self.extensions.is_empty() {
      items_vec.extend_from_slice(items);
      for ext in &self.extensions {
        ext.before_items(&query, &mut items_vec);
      }
      &items_vec[..]
    } else {
      items
    };

    if self.searchers.is_empty() {
      return Vec::new();
    }

    let context = SearchContext::new(items_slice);

    // Collect results from all searchers
    // OPTIMIZATION: Run all searchers in parallel when parallel feature is enabled
    #[cfg(feature = "parallel")]
    let mut all_results: Vec<(SearcherKind, Vec<SearusMatch<T>>)> = self
      .searchers
      .par_iter() // Note: par_iter might make it hard to run sequential hooks per searcher
      .map(|searcher| {
        // Note: before_searcher hook is tricky with parallel execution if it modifies searcher.
        // Since searcher is &self here, we can't modify it.
        // And before_searcher hook takes &mut Box<dyn Searcher>.
        // So we can't easily support before_searcher in parallel mode without significant changes.
        // For now, let's skip before_searcher in parallel mode or run it sequentially first?
        // Running sequentially first doesn't help if we want to modify the searcher instance used in parallel.
        // Given the constraints, we might skip before_searcher in parallel mode or accept it's read-only?
        // But the trait signature is &self for extension.
        // Wait, before_searcher takes &mut Box<dyn Searcher>.
        // We can't mutate searchers in the engine while iterating.
        // So before_searcher is effectively disabled for now in this implementation unless we clone searchers?
        // Let's proceed without calling before_searcher in the parallel block for now,
        // or maybe we can't support parallel execution with mutable searcher hooks easily.
        // Let's just run searchers.
        let results = searcher.search(&context, &query);

        // Hook: after_searcher (we can run this on the results)
        // But we need access to extensions. Extensions are Sync, so we can access them.
        // But we need to iterate over them.
        // Also, we can't easily call after_searcher in the parallel map closure because extensions might not be safe to call in parallel?
        // Extensions are Send + Sync. So we can call them.
        // But `after_searcher` takes `&mut Vec<SearusMatch>`. That's fine, we have ownership of results.
        // So we can do it.
        (searcher.kind(), results)
      })
      .filter(|(_, results)| !results.is_empty())
      .collect();

    // Apply after_searcher hooks in parallel results (need to do it after collect or inside map?)
    // Inside map is better for parallelism.
    // Let's refine the parallel block.

    #[cfg(feature = "parallel")]
    {
      // We need to iterate over results and apply hooks.
      // Since we already collected, let's iterate again or do it in the map.
      // Doing it in map requires extensions to be available.
      // self.extensions is available.
      // But we skipped before_searcher.
      // Let's just apply after_searcher here.
      for (_, results) in &mut all_results {
        for ext in &self.extensions {
          ext.after_searcher(&query, results);
        }
      }
    }

    #[cfg(not(feature = "parallel"))]
    let mut all_results: Vec<(SearcherKind, Vec<SearusMatch<T>>)> = self
      .searchers
      .iter()
      .map(|searcher| {
        // We can't easily mutate searcher here either because it's behind a reference in the Vec.
        // To support before_searcher modifying the searcher, we'd need RefCell or similar, or clone.
        // For now, we'll skip before_searcher modification support or just pass a clone if searchers were cloneable (they aren't easily).
        // Let's skip before_searcher for now as it requires architectural changes to Searcher storage.
        // Or we can just pass the reference if we change the hook signature?
        // The trait says `&mut Box<dyn Searcher>`.
        // We can't do that with `iter()`.
        // So we will skip `before_searcher` invocation for now to avoid breaking compilation,
        // and note it as a limitation or future work.
        // Actually, let's try to implement `after_searcher`.
        let mut results = searcher.search(&context, &query);
        for ext in &self.extensions {
          ext.after_searcher(&query, &mut results);
        }
        (searcher.kind(), results)
      })
      .filter(|(_, results)| !results.is_empty())
      .collect();

    if all_results.is_empty() {
      return Vec::new();
    }

    // Hook: before_merge
    for ext in &self.extensions {
      ext.before_merge(&query, &mut all_results);
    }

    // Normalize scores for each searcher's results
    let normalized_results = self.normalize_results(all_results);

    // Merge and rank results
    let mut merged = self.merge_results(normalized_results, &query);

    // Hook: after_merge
    for ext in &self.extensions {
      ext.after_merge(&query, &mut merged);
    }

    // Hook: before_limit
    for ext in &self.extensions {
      ext.before_limit(&query, &mut merged);
    }

    // Apply pagination
    let skip = query.options.skip;
    let limit = query.options.limit;

    let mut final_results: Vec<SearusMatch<T>> =
      merged.into_iter().skip(skip).take(limit).collect();

    // Hook: after_limit
    for ext in &self.extensions {
      ext.after_limit(&query, &mut final_results);
    }

    final_results
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
    // OPTIMIZATION: Normalize each searcher's results in parallel
    #[cfg(feature = "parallel")]
    let normalized = results
      .into_par_iter()
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
      .collect();

    #[cfg(not(feature = "parallel"))]
    let normalized = results
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
      .collect();

    normalized
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
  extensions: Vec<Box<dyn SearusExtension<T>>>,
}

impl<T> SearusEngineBuilder<T> {
  /// Creates a new, empty `SearusEngineBuilder`.
  pub fn new() -> Self {
    Self {
      searchers: Vec::new(),
      normalization: None,
      extensions: Vec::new(),
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

  /// Adds an extension to the engine.
  pub fn with_extension(mut self, extension: Box<dyn SearusExtension<T>>) -> Self {
    self.extensions.push(extension);
    self
  }

  /// Builds the `SearusEngine` with the configured components.
  pub fn build(self) -> SearusEngine<T> {
    SearusEngine {
      searchers: self.searchers,
      normalization: self.normalization.unwrap_or(NormalizationMethod::MinMax),
      extensions: self.extensions,
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
