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
/// `SearusEngine` is the central component of the library, responsible for managing a
/// collection of `Searcher` plugins. It orchestrates the entire search process, from
/// dispatching queries to normalizing and merging results from different sources.
///
/// The engine is highly extensible, allowing for custom search logic via the `Searcher`
/// trait and lifecycle hooks through the `SearusExtension` trait. This design enables
/// complex, multi-modal search strategies that can be tailored to specific needs.
///
/// Create a `SearusEngine` using the [`SearusEngineBuilder`].
///
/// # Examples
///
/// ```rust
/// use searus::prelude::*;
/// use searus::searchers::SemanticSearch;
///
/// // 1. Define your data type
/// #[derive(Debug, Clone, serde::Serialize)]
/// struct Product {
///     id: u32,
///     name: String,
///     description: String,
/// }
///
/// // 2. Configure rules and searcher
/// let rules = SemanticRules::builder()
///     .field("name", FieldRule::bm25().priority(2))
///     .build();
/// let searcher = SemanticSearch::new(rules);
///
/// // 3. Build the engine
/// let engine: SearusEngine<Product> = SearusEngine::builder()
///     .with(Box::new(searcher))
///     .normalization(NormalizationMethod::MinMax)
///     .build();
/// ```
pub struct SearusEngine<T> {
  /// The collection of registered searcher plugins.
  searchers: Vec<Box<dyn Searcher<T>>>,
  /// The method used to normalize scores from different searchers.
  normalization: NormalizationMethod,
  /// The collection of registered extensions that hook into the search lifecycle.
  extensions: Vec<Box<dyn SearusExtension<T>>>,
}

impl<T: Searchable> SearusEngine<T> {
  /// Creates a new `SearusEngineBuilder` to construct an engine.
  ///
  /// # Returns
  ///
  /// A `SearusEngineBuilder` instance to configure a new engine.
  pub fn builder() -> SearusEngineBuilder<T> {
    SearusEngineBuilder::new()
  }

  /// Searches for items using all registered searchers and merges the results.
  ///
  /// The search process follows a well-defined lifecycle, with hooks for `SearusExtension`
  /// traits at various stages.
  ///
  /// ## Search Lifecycle
  ///
  /// 1.  **Query Initialization**: The initial `Query` is received.
  /// 2.  **`before_query` Hook**: Extensions can modify the `Query` before it's sent to any searcher.
  ///     For example, an extension could rewrite query text (e.g., "ml" -> "machine learning").
  /// 3.  **`before_items` Hook**: Extensions can modify the collection of items to be searched.
  ///     This allows for dynamically adding or removing items from the search context.
  /// 4.  **Parallel Search Execution**: The query is dispatched to all registered `Searcher` instances.
  ///     If the `parallel` feature is enabled, this happens concurrently.
  /// 5.  **`after_searcher` Hook**: After each searcher returns its results, extensions can modify
  ///     the list of matches (e.g., boosting scores, filtering).
  /// 6.  **`before_merge` Hook**: Extensions can inspect or modify the collected results from all
  ///     searchers before they are normalized and merged.
  /// 7.  **Score Normalization**: Scores from each searcher are normalized to a common scale (e.g., 0.0 to 1.0)
  ///     using the configured `NormalizationMethod`.
  /// 8.  **Result Merging**: The normalized results are merged. If multiple searchers match the same
  ///     item, their scores are combined using a weighted sum based on `SearchOptions`.
  /// 9.  **`after_merge` Hook**: Extensions can modify the final, merged list of results before sorting.
  /// 10. **Sorting**: The merged list is sorted by score in descending order.
  /// 11. **`before_limit` Hook**: Extensions can access the sorted list before pagination is applied.
  /// 12. **Pagination**: `skip` and `limit` from `SearchOptions` are applied.
  /// 13. **`after_limit` Hook**: The final, paginated list of results can be modified by extensions.
  /// 14. **Return**: The final `Vec<SearusMatch<T>>` is returned.
  ///
  /// # Arguments
  ///
  /// * `items` - A slice of items to be searched. These items must implement `Searchable`.
  /// * `query` - The search query containing text, tags, filters, and other options.
  ///
  /// # Returns
  ///
  /// A `Vec<SearusMatch<T>>` containing the final, ranked, and paginated search results.
  ///
  /// # Examples
  ///
  /// ```rust
  /// # use searus::prelude::*;
  /// # use searus::searchers::SemanticSearch;
  /// # #[derive(Debug, Clone, serde::Serialize)]
  /// # struct Product { name: String }
  /// # let rules = SemanticRules::builder().field("name", FieldRule::exact()).build();
  /// # let searcher = SemanticSearch::new(rules);
  /// # let engine = SearusEngine::builder().with(Box::new(searcher)).build();
  /// # let products = vec![Product { name: "Phone".into() }];
  ///
  /// let query = Query::builder()
  ///     .text("phone")
  ///     .options(SearchOptions::default().limit(5))
  ///     .build();
  ///
  /// let results = engine.search(&products, &query);
  ///
  /// for result in results {
  ///     println!("Found item: {:?} with score {}", result.item, result.score);
  /// }
  /// ```
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
    let mut items_vec = if !self.extensions.is_empty() {
      items.to_vec()
    } else {
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
    #[cfg(feature = "parallel")]
    let mut all_results: Vec<(SearcherKind, Vec<SearusMatch<T>>)> = self
      .searchers
      .par_iter()
      .map(|searcher| {
        let mut results = searcher.search(&context, &query);
        for ext in &self.extensions {
          ext.after_searcher(&query, &mut results);
        }
        (searcher.kind(), results)
      })
      .filter(|(_, results)| !results.is_empty())
      .collect();

    #[cfg(not(feature = "parallel"))]
    let mut all_results: Vec<(SearcherKind, Vec<SearusMatch<T>>)> = self
      .searchers
      .iter()
      .map(|searcher| {
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

    // Sort before applying limit
    merged.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

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
    let iter = results.into_par_iter();
    #[cfg(not(feature = "parallel"))]
    let iter = results.into_iter();

    iter
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
        let item_id = m.id;

        let entry = merged.entry(item_id).or_insert_with(|| SearusMatch {
          id: item_id,
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

    // Convert the map of merged items to a Vec. Sorting is done later.
    merged.into_values().collect()
  }
}

/// A builder for creating `SearusEngine` instances.
///
/// The builder pattern provides a fluent and convenient way to configure and
/// construct a `SearusEngine` with the desired searchers, extensions, and
/// normalization method.
///
/// # Examples
///
/// ```
/// use searus::prelude::*;
/// use searus::searchers::{SemanticSearch, TaggedSearch};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// struct Post {
///     title: String,
///     content: String,
///     tags: Vec<String>,
/// }
///
/// // 1. Define semantic rules for text search.
/// let semantic_rules = SemanticRules::builder()
///     .field("title", FieldRule::bm25().priority(2))
///     .field("content", FieldRule::tokenized().priority(1))
///     .build();
///
/// // 2. Create searcher instances.
/// let semantic_searcher = SemanticSearch::new(semantic_rules);
/// let tag_searcher = TaggedSearch::new();
///
/// // 3. Build the engine with multiple searchers.
/// let engine = SearusEngine::builder()
///     .with(Box::new(semantic_searcher))
///     .with(Box::new(tag_searcher))
///     .normalization(NormalizationMethod::MinMax) // Optional: default is MinMax
///     .build();
///
/// // 4. The engine is now ready to perform searches.
/// let posts = vec![
///     Post {
///         title: "Rust Concurrency".to_string(),
///         content: "A deep dive into fearless concurrency.".to_string(),
///         tags: vec!["rust".to_string(), "tutorial".to_string()],
///     }
/// ];
///
/// let query = Query::builder()
///     .text("rust")
///     .tags(vec!["tutorial".to_string()])
///     .options(
///         SearchOptions::default()
///             .weight(SearcherKind::Semantic, 0.7) // Give semantic search more weight
///             .weight(SearcherKind::Tags, 0.3)     // Give tag search less weight
///     )
///     .build();
///
/// let results = engine.search(&posts, &query);
/// assert!(!results.is_empty());
/// ```
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
  /// Searchers are the core components that perform the actual search logic.
  /// They are added as boxed traits to allow for different underlying implementations.
  ///
  /// # Arguments
  ///
  /// * `searcher` - A `Box<dyn Searcher<T>>` instance.
  pub fn with(mut self, searcher: Box<dyn Searcher<T>>) -> Self {
    self.searchers.push(searcher);
    self
  }

  /// Sets the score normalization method for the engine.
  ///
  /// If not set, `NormalizationMethod::MinMax` is used by default.
  ///
  /// # Arguments
  ///
  /// * `method` - The `NormalizationMethod` to use.
  pub fn normalization(mut self, method: NormalizationMethod) -> Self {
    self.normalization = Some(method);
    self
  }

  /// Adds an extension to the engine.
  ///
  /// Extensions provide a way to hook into the search lifecycle to modify
  /// queries, items, or results.
  ///
  /// # Arguments
  ///
  /// * `extension` - A `Box<dyn SearusExtension<T>>` instance.
  pub fn with_extension(mut self, extension: Box<dyn SearusExtension<T>>) -> Self {
    self.extensions.push(extension);
    self
  }

  /// Builds the `SearusEngine` with the configured components.
  ///
  /// # Returns
  ///
  /// A new `SearusEngine<T>` instance.
  pub fn build(self) -> SearusEngine<T> {
    SearusEngine {
      searchers: self.searchers,
      normalization: self.normalization.unwrap_or(NormalizationMethod::MinMax),
      extensions: self.extensions,
    }
  }
}

/// Defines the methods for normalizing scores from different searchers.
///
/// Normalization is crucial when combining results from multiple searchers,
/// as each may produce scores on a different scale. By normalizing scores to a
/// common range (like 0.0 to 1.0), they can be meaningfully compared and combined.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizationMethod {
  /// **Min-Max Normalization**: Scales scores to a `[0, 1]` range.
  ///
  /// This is the most common method. It preserves the relative ranking of items
  /// within a searcher's result set.
  ///
  /// The formula is: `(score - min_score) / (max_score - min_score)`.
  /// If all scores are the same, they are all normalized to `1.0`.
  ///
  /// # Example
  ///
  /// Use this when combining searchers that produce scores in arbitrary ranges,
  /// like BM25 (unbounded positive) and Fuzzy (0.0 - 1.0).
  MinMax,

  /// **Inverse Distance Normalization**: Converts distance scores to similarity scores.
  ///
  /// This method is useful when a searcher returns a "distance" where lower scores
  /// are better (e.g., vector search distance). It transforms the distance into a
  /// similarity score where higher is better.
  ///
  /// The formula is: `1.0 / (1.0 + distance)`.
  /// This ensures that a distance of 0.0 becomes a similarity of 1.0, and larger
  /// distances result in progressively smaller similarity scores.
  ///
  /// # Example
  ///
  /// Use this for vector searchers that return Euclidean distance or Cosine distance.
  InverseDistance,
}
