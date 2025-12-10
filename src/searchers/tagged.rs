//! A `Searcher` implementation for matching tags.

use crate::context::SearchContext;
use crate::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

#[cfg(feature = "parallel")]
pub trait TaggedSearchable: serde::Serialize + Clone + Send + Sync {}
#[cfg(feature = "parallel")]
impl<T: serde::Serialize + Clone + Send + Sync> TaggedSearchable for T {}

#[cfg(not(feature = "parallel"))]
pub trait TaggedSearchable: serde::Serialize + Clone {}
#[cfg(not(feature = "parallel"))]
impl<T: serde::Serialize + Clone> TaggedSearchable for T {}

/// Represents a tag and its relationships to other tags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagNode {
  pub tag: String,
  /// Map of related tags to their relationship strength (0 < strength <= 1)
  pub relationships: HashMap<String, f32>,
}

/// A Tag Relationship Tree that defines hierarchical/semantic relationships between tags.
///
/// TRT enables expansion of tag queries to include related tags with weighted scoring
/// based on relationship strength and distance in the tree.
#[derive(Debug, Clone, Default)]
pub struct TagRelationshipTree {
  /// Internal representation: tag -> (related_tag -> strength)
  nodes: HashMap<String, HashMap<String, f32>>,
}

impl TagRelationshipTree {
  /// Creates a new Tag Relationship Tree from a vector of tag nodes.
  pub fn new(nodes: Vec<TagNode>) -> Self {
    let mut tree = HashMap::new();
    for node in nodes {
      tree.insert(node.tag, node.relationships);
    }
    Self { nodes: tree }
  }

  /// Expands query tags using the relationship tree up to a specified depth.
  ///
  /// Returns a map of all reachable tags to their accumulated relationship strength.
  /// The strength is calculated as the product of all edge strengths along the path
  /// from the original query tag.
  ///
  /// # Arguments
  ///
  /// * `query_tags` - The original tags to expand
  /// * `max_depth` - Maximum depth to traverse (0 = no expansion, only original tags)
  ///
  /// # Returns
  ///
  /// HashMap mapping expanded tags to their relationship strengths (0 < strength <= 1)
  pub fn expand_tags(&self, query_tags: &[String], max_depth: usize) -> HashMap<String, f32> {
    let mut expanded = HashMap::new();

    // Start with original query tags at full strength
    for tag in query_tags {
      expanded.insert(tag.to_lowercase(), 1.0);
    }

    if max_depth == 0 || self.nodes.is_empty() {
      return expanded;
    }

    // BFS traversal for each query tag
    for query_tag in query_tags {
      let query_tag_lower = query_tag.to_lowercase();
      let mut queue = VecDeque::new();
      let mut visited = HashSet::new();

      queue.push_back((query_tag_lower.clone(), 0, 1.0)); // (tag, depth, strength)
      visited.insert(query_tag_lower.clone());

      while let Some((current_tag, depth, current_strength)) = queue.pop_front() {
        if depth >= max_depth {
          continue;
        }

        // Find relationships for current tag
        if let Some(relationships) = self.nodes.get(&current_tag) {
          for (related_tag, edge_strength) in relationships {
            let related_tag_lower = related_tag.to_lowercase();
            let new_strength = current_strength * edge_strength;

            // Update or insert the expanded tag with maximum strength found
            expanded
              .entry(related_tag_lower.clone())
              .and_modify(|e| *e = e.max(new_strength))
              .or_insert(new_strength);

            // Continue BFS if not visited at this depth
            if !visited.contains(&related_tag_lower) {
              visited.insert(related_tag_lower.clone());
              queue.push_back((related_tag_lower, depth + 1, new_strength));
            }
          }
        }
      }
    }

    expanded
  }
}

/// A searcher that finds items by matching tags.
///
/// `TaggedSearch` is designed to filter or score items based on a list of tags.
/// It works by extracting tags from a specified field in the items and comparing
/// them against the tags provided in the search query.
///
/// Optionally supports Tag Relationship Tree (TRT) for hierarchical tag expansion.
pub struct TaggedSearch {
  /// The name of the field that contains the tags in the item.
  tag_field: String,
  /// Optional Tag Relationship Tree for semantic tag expansion
  trt: Option<TagRelationshipTree>,
}

impl TaggedSearch {
  /// Creates a new `TaggedSearch` instance with the default tag field ("tags").
  pub fn new() -> Self {
    Self {
      tag_field: "tags".to_string(),
      trt: None,
    }
  }

  /// Creates a new `TaggedSearch` instance with a custom tag field.
  ///
  /// # Arguments
  ///
  /// * `tag_field` - The name of the field to extract tags from.
  pub fn with_field(tag_field: impl Into<String>) -> Self {
    Self {
      tag_field: tag_field.into(),
      trt: None,
    }
  }

  /// Adds a Tag Relationship Tree to enable hierarchical tag expansion.
  ///
  /// # Arguments
  ///
  /// * `trt` - The Tag Relationship Tree to use for query expansion
  pub fn with_trt(mut self, trt: TagRelationshipTree) -> Self {
    self.trt = Some(trt);
    self
  }

  /// Extracts a list of tags from a specified field in a serializable item.
  ///
  /// This helper function serializes the item to a `serde_json::Value` and
  /// expects the specified field to contain an array of strings.
  fn extract_tags<T>(item: &T, field: &str) -> Vec<String>
  where
    T: serde::Serialize,
  {
    let value = match serde_json::to_value(item) {
      Ok(v) => v,
      Err(_) => return Vec::new(),
    };

    let tags_value = match value.get(field) {
      Some(v) => v,
      None => return Vec::new(),
    };

    match tags_value {
      Value::Array(arr) => arr
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect(),
      _ => Vec::new(),
    }
  }
}

impl Default for TaggedSearch {
  fn default() -> Self {
    Self::new()
  }
}

impl<T> Searcher<T> for TaggedSearch
where
  T: TaggedSearchable,
{
  fn kind(&self) -> SearcherKind {
    SearcherKind::Tags
  }

  /// Performs a search by matching the query tags against the tags of the items.
  ///
  /// This method checks each item to see if its tags (extracted from the
  /// configured `tag_field`) contain any of the tags specified in `query.tags`.
  /// The matching is case-insensitive.
  ///
  /// The raw score for a matched item is calculated as the ratio of the number
  /// of matching tags to the total number of tags in the query. For example, if
  /// the query has 4 tags and the item matches 2 of them, the score will be 0.5.
  fn search(&self, context: &SearchContext<T>, query: &Query) -> Vec<SearusMatch<T>> {
    let items = context.items;
    let query_tags = match &query.tags {
      Some(tags) => tags,
      None => return Vec::new(),
    };

    if query_tags.is_empty() {
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
        .filter_map(|(index, item)| self.match_entity(item, index, query, query_tags))
        .collect();

      let mut results = Vec::with_capacity(matches.len());
      results.extend(matches);
      results
    };

    #[cfg(not(feature = "parallel"))]
    let mut results: Vec<SearusMatch<T>> = {
      // OPTIMIZATION: Pre-allocate with estimated capacity
      let mut results = Vec::with_capacity(items.len() / 5); // Assume ~20% tag match rate
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
          .filter_map(|(index, item)| self.match_entity(item, index, query, query_tags)),
      );
      results
    };

    // Sort results by score in descending order.
    self.sort_results(&mut results);

    results
  }
}

impl TaggedSearch {
  /// Match a single entity against the query.
  pub fn match_entity<T>(
    &self,
    item: &T,
    index: usize,
    query: &Query,
    query_tags: &[String],
  ) -> Option<SearusMatch<T>>
  where
    T: TaggedSearchable,
  {
    let item_tags = Self::extract_tags(item, &self.tag_field);
    if item_tags.is_empty() {
      return None;
    }

    // Check if TRT expansion is enabled
    let expanded_tags = if let (Some(trt), Some(depth)) = (&self.trt, query.options.trt_depth) {
      if depth > 0 {
        trt.expand_tags(query_tags, depth)
      } else {
        // No expansion, just original tags at strength 1.0
        query_tags.iter().map(|t| (t.to_lowercase(), 1.0)).collect()
      }
    } else {
      // No TRT, just original tags at strength 1.0
      query_tags.iter().map(|t| (t.to_lowercase(), 1.0)).collect()
    };

    // OPTIMIZATION: Pre-allocate with expected capacity
    let mut matched_tags = Vec::with_capacity(query_tags.len().min(item_tags.len()));
    let mut total_strength = 0.0;
    let mut max_strength: f32 = 0.0;

    // Match item tags against expanded tags
    for item_tag in &item_tags {
      let item_tag_lower = item_tag.to_lowercase();
      if let Some(&strength) = expanded_tags.get(&item_tag_lower) {
        matched_tags.push(item_tag.clone());
        total_strength += strength;
        max_strength = max_strength.max(strength);
      }
    }

    // If there are any matches, create a SearusMatch
    if !matched_tags.is_empty() {
      // Score calculation:
      // - Base score is the proportion of matched query tags
      // - Weighted by the average relationship strength of matched tags
      let base_score = matched_tags.len() as f32 / query_tags.len() as f32;
      let avg_strength = total_strength / matched_tags.len() as f32;
      let score = base_score * avg_strength;

      let mut m = SearusMatch::new(item.clone(), score, index);
      m.details.push(SearchDetail::Tag {
        matched_tags,
        total_tags: item_tags.len(),
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

  #[cfg(not(feature = "parallel"))]
  pub fn sort_results<T>(&self, results: &mut Vec<SearusMatch<T>>) {
    results.sort_by(|a, b| {
      b.score
        .partial_cmp(&a.score)
        .unwrap_or(std::cmp::Ordering::Equal)
    });
  }
}
