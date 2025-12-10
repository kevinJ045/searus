//! A `Searcher` implementation for matching tags.

use crate::prelude::*;
use serde_json::Value;

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

/// A searcher that finds items by matching tags.
///
/// `TaggedSearch` is designed to filter or score items based on a list of tags.
/// It works by extracting tags from a specified field in the items and comparing
/// them against the tags provided in the search query.
pub struct TaggedSearch {
  /// The name of the field that contains the tags in the item.
  tag_field: String,
}

impl TaggedSearch {
  /// Creates a new `TaggedSearch` instance with the default tag field ("tags").
  pub fn new() -> Self {
    Self {
      tag_field: "tags".to_string(),
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
    }
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
  fn search(&self, query: &Query, items: &[T]) -> Vec<SearusMatch<T>> {
    let query_tags = match &query.tags {
      Some(tags) => tags,
      None => return Vec::new(),
    };

    if query_tags.is_empty() {
      return Vec::new();
    }

    #[cfg(feature = "parallel")]
    let mut results: Vec<SearusMatch<T>> = items
      .par_iter()
      .enumerate()
      .filter_map(|(index, item)| self.match_entity(item, index, query, query_tags))
      .collect();

    #[cfg(not(feature = "parallel"))]
    let mut results: Vec<SearusMatch<T>> = items
      .iter()
      .enumerate()
      .filter_map(|(index, item)| self.match_entity(item, index, query, query_tags))
      .collect();

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
    _query: &Query,
    query_tags: &[String],
  ) -> Option<SearusMatch<T>>
  where
    T: TaggedSearchable,
  {
    let item_tags = Self::extract_tags(item, &self.tag_field);
    if item_tags.is_empty() {
      return None;
    }

    // Find all tags that match between the query and the item.
    let mut matched_tags = Vec::new();
    for query_tag in query_tags {
      if item_tags.iter().any(|t| t.eq_ignore_ascii_case(query_tag)) {
        matched_tags.push(query_tag.clone());
      }
    }

    // If there are any matches, create a SearusMatch.
    if !matched_tags.is_empty() {
      // The score is the proportion of matched query tags.
      let score = matched_tags.len() as f32 / query_tags.len() as f32;

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
