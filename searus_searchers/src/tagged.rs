//! Tagged search implementation.

use searus_core::prelude::*;
use serde_json::Value;

/// Tag-based searcher.
pub struct TaggedSearch {
    /// Field name containing tags (default: "tags").
    tag_field: String,
}

impl TaggedSearch {
    /// Create a new tagged searcher with default field name.
    pub fn new() -> Self {
        Self {
            tag_field: "tags".to_string(),
        }
    }

    /// Create a new tagged searcher with custom field name.
    pub fn with_field(tag_field: impl Into<String>) -> Self {
        Self {
            tag_field: tag_field.into(),
        }
    }

    /// Extract tags from an item.
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
    T: serde::Serialize + Clone,
{
    fn kind(&self) -> SearcherKind {
        SearcherKind::Tags
    }

    fn search(&self, query: &Query, items: &[T]) -> Vec<SearusMatch<T>> {
        let query_tags = match &query.tags {
            Some(tags) => tags,
            None => return Vec::new(),
        };

        if query_tags.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();

        for item in items {
            let item_tags = Self::extract_tags(item, &self.tag_field);
            if item_tags.is_empty() {
                continue;
            }

            // Count matching tags
            let mut matched_tags = Vec::new();
            for query_tag in query_tags {
                if item_tags.iter().any(|t| t.eq_ignore_ascii_case(query_tag)) {
                    matched_tags.push(query_tag.clone());
                }
            }

            if !matched_tags.is_empty() {
                let score = matched_tags.len() as f32 / query_tags.len() as f32;
                
                let mut m = SearusMatch::new(item.clone(), score);
                m.details.push(SearchDetail::Tag {
                    matched_tags,
                    total_tags: item_tags.len(),
                });

                results.push(m);
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        results
    }
}
