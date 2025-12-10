//! Core data types for the Searus search engine.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "parallel")]
pub trait Searchable: Send + Sync {}
#[cfg(feature = "parallel")]
impl<T: Send + Sync> Searchable for T {}

#[cfg(not(feature = "parallel"))]
pub trait Searchable {}
#[cfg(not(feature = "parallel"))]
impl<T> Searchable for T {}

/// Type alias for entity identifiers.
///
/// Using a dedicated type alias makes it easier to change the underlying type
/// of the identifier in the future if needed. It also improves readability.
pub type EntityId = String;

/// A search match result containing the matched item, score, and metadata.
///
/// This struct represents a single item returned from a search query. It includes
/// the item itself, a normalized score indicating the relevance of the match,
/// and detailed metadata about why this item was matched.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg(not(feature = "parallel"))]
pub struct SearusMatch<T> {
  /// The matched item or entity that was found in the search.
  pub item: T,
  /// The final normalized score, ranging from 0.0 to 1.0, where a higher score
  /// indicates a better match. This score is often a blended result from multiple
  /// underlying search mechanisms.
  pub score: f32,
  /// An optional breakdown of scores per field, providing explainability for
  /// why the item received its final score. For example, in a text search, this
  /// could show the scores for matches in the 'title' vs. 'description' fields.
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  pub field_scores: HashMap<String, f32>,
  /// A list of searcher-specific details that provide low-level metadata about
  /// the match. This can include information about which terms matched, the
  /// vector similarity, or other details from the specific searcher that
  /// produced this match.
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub details: Vec<SearchDetail>,

  pub id: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg(feature = "parallel")]
pub struct SearusMatch<T>
where
  T: Send + Sync,
{
  /// The matched item or entity that was found in the search.
  pub item: T,
  /// The final normalized score, ranging from 0.0 to 1.0, where a higher score
  /// indicates a better match. This score is often a blended result from multiple
  /// underlying search mechanisms.
  pub score: f32,
  /// An optional breakdown of scores per field, providing explainability for
  /// why the item received its final score. For example, in a text search, this
  /// could show the scores for matches in the 'title' vs. 'description' fields.
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  pub field_scores: HashMap<String, f32>,
  /// A list of searcher-specific details that provide low-level metadata about
  /// the match. This can include information about which terms matched, the
  /// vector similarity, or other details from the specific searcher that
  /// produced this match.
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub details: Vec<SearchDetail>,

  pub id: usize,
}

impl<T: Searchable> SearusMatch<T> {
  /// Creates a new search match with a given item and score.
  ///
  /// This is a convenience method for creating a `SearusMatch` with default
  /// empty values for `field_scores` and `details`.
  pub fn new(item: T, score: f32, id: usize) -> Self {
    Self {
      id,
      item,
      score,
      field_scores: HashMap::new(),
      details: Vec::new(),
    }
  }

  /// Adds a field score to the match.
  ///
  /// This is useful for building up the `field_scores` map in a chained manner.
  pub fn with_field_score(mut self, field: impl Into<String>, score: f32) -> Self {
    self.field_scores.insert(field.into(), score);
    self
  }

  /// Adds a search detail to the match.
  ///
  /// This is useful for building up the `details` vector in a chained manner.
  pub fn with_detail(mut self, detail: SearchDetail) -> Self {
    self.details.push(detail);
    self
  }
}

/// Searcher-specific metadata that provides detailed insight into a match.
///
/// Each variant of this enum corresponds to a specific type of searcher and
/// contains information that is relevant to that searcher's matching logic.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SearchDetail {
  /// Details for a semantic text match from a searcher like BM25.
  #[cfg(feature = "semantic")]
  Semantic {
    /// The specific terms that were matched within the text.
    matched_terms: Vec<String>,
    /// The field in which the match occurred (e.g., "title", "content").
    field: String,
    /// The weight of this particular match.
    weight: f32,
  },
  /// Details for a vector similarity search.
  Vector {
    /// The distance between the query vector and the item's vector.
    distance: f32,
    /// The calculated similarity score, often derived from the distance.
    similarity: f32,
  },
  /// Details for a tag-based match.
  #[cfg(feature = "tagged")]
  Tag {
    /// The tags that matched the query.
    matched_tags: Vec<String>,
    /// The total number of tags the item has.
    total_tags: usize,
  },
  /// Details for a fuzzy (approximate) string match.
  #[cfg(feature = "fuzzy")]
  Fuzzy {
    /// The term from the item that was matched.
    matched_term: String,
    /// The original query term that this match corresponds to.
    original_term: String,
    /// The similarity score between the matched term and the original term.
    similarity: f32,
  },
  /// Details for an image-based similarity match.
  Image {
    /// The similarity score between the query image and the item's image.
    similarity: f32,
  },
}

/// Represents a search query that can combine multiple search modes.
///
/// A `Query` can include a text query, a vector for similarity search, tags,
/// image data, and filters. This allows for complex, multi-faceted searches
/// to be expressed in a single structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Query {
  /// The text to be used for semantic or keyword-based search.
  pub text: Option<String>,
  /// A pre-computed embedding vector for vector similarity search.
  pub vector: Option<Vec<f32>>,
  /// A list of tags to filter or score results by.
  pub tags: Option<Vec<String>>,
  /// Image data to be used for image similarity search.
  pub image: Option<ImageData>,
  /// A filter expression to apply to the search results, allowing for
  /// structured filtering based on item attributes.
  pub filters: Option<crate::filter::FilterExpr>,
  /// Additional options for the search, such as pagination, timeouts, and
  /// searcher-specific weights.
  #[serde(default)]
  pub options: SearchOptions,
}

impl Query {
  /// Creates a new `QueryBuilder` to construct a `Query` in a chained manner.
  pub fn builder() -> QueryBuilder {
    QueryBuilder::default()
  }
}

/// A builder for creating `Query` instances.
///
/// The builder pattern provides a more ergonomic way to construct complex
/// `Query` objects.
#[derive(Debug, Default)]
pub struct QueryBuilder {
  text: Option<String>,
  vector: Option<Vec<f32>>,
  tags: Option<Vec<String>>,
  image: Option<ImageData>,
  filters: Option<crate::filter::FilterExpr>,
  options: SearchOptions,
}

impl QueryBuilder {
  /// Sets the text component of the query.
  pub fn text(mut self, text: impl Into<String>) -> Self {
    self.text = Some(text.into());
    self
  }

  /// Sets the vector component of the query.
  pub fn vector(mut self, vector: Vec<f32>) -> Self {
    self.vector = Some(vector);
    self
  }

  /// Sets the tags component of the query.
  pub fn tags(mut self, tags: Vec<String>) -> Self {
    self.tags = Some(tags);
    self
  }

  /// Sets the image component of the query.
  pub fn image(mut self, image: ImageData) -> Self {
    self.image = Some(image);
    self
  }

  /// Sets the filter expression for the query.
  pub fn filters(mut self, filters: crate::filter::FilterExpr) -> Self {
    self.filters = Some(filters);
    self
  }

  /// Sets the search options for the query.
  pub fn options(mut self, options: SearchOptions) -> Self {
    self.options = options;
    self
  }

  /// Builds the final `Query` object.
  pub fn build(self) -> Query {
    Query {
      text: self.text,
      vector: self.vector,
      tags: self.tags,
      image: self.image,
      filters: self.filters,
      options: self.options,
    }
  }
}

/// Represents image data for an image-based search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
  /// The raw byte content of the image file.
  pub bytes: Vec<u8>,
  /// The MIME type of the image, e.g., "image/png" or "image/jpeg".
  pub mime_type: Option<String>,
  /// The width of the image in pixels, if known.
  pub width: Option<u32>,
  /// The height of the image in pixels, if known.
  pub height: Option<u32>,
}

/// Defines options for controlling a search operation.
///
/// This includes settings for pagination, timeouts, and weighting of different
/// searcher types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
  /// The number of results to skip from the beginning of the result set.
  /// Used for pagination.
  #[serde(default)]
  pub skip: usize,
  /// The maximum number of results to return in this query.
  #[serde(default = "default_limit")]
  pub limit: usize,
  /// An optional timeout in milliseconds for the search operation. If the
  /// search takes longer than this, it may be aborted. A value of 0 means
  /// no timeout.
  #[serde(default)]
  pub timeout_ms: u64,
  /// A map of weights to apply to the scores from different types of searchers.
  /// This allows for fine-tuning the relevance blending between, for example,
  /// semantic search and tag-based search.
  #[serde(default)]
  pub weights: HashMap<SearcherKind, f32>,
}

/// Returns the default limit for search results.
fn default_limit() -> usize {
  20
}

impl Default for SearchOptions {
  /// Creates a default set of search options.
  fn default() -> Self {
    Self {
      skip: 0,
      limit: default_limit(),
      timeout_ms: 0,
      weights: HashMap::new(),
    }
  }
}

impl SearchOptions {
  /// Sets the `skip` value for pagination.
  pub fn skip(mut self, skip: usize) -> Self {
    self.skip = skip;
    self
  }

  /// Sets the `limit` value for the maximum number of results.
  pub fn limit(mut self, limit: usize) -> Self {
    self.limit = limit;
    self
  }

  /// Sets the timeout in milliseconds for the search.
  pub fn timeout_ms(mut self, timeout_ms: u64) -> Self {
    self.timeout_ms = timeout_ms;
    self
  }

  /// Sets a weight for a specific kind of searcher.
  pub fn weight(mut self, kind: SearcherKind, weight: f32) -> Self {
    self.weights.insert(kind, weight);
    self
  }
}

/// An enumeration of the different kinds of searchers available.
///
/// This is used to identify and configure specific searcher implementations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SearcherKind {
  /// A searcher based on semantic text analysis (e.g., BM25).
  #[cfg(feature = "semantic")]
  Semantic,
  /// A searcher based on vector similarity.
  Vector,
  /// A searcher that matches based on tags.
  #[cfg(feature = "tagged")]
  Tags,
  /// A searcher for image similarity.
  Image,
  /// A searcher for fuzzy (approximate) string matching.
  #[cfg(feature = "fuzzy")]
  Fuzzy,
  /// A searcher for numerical or date ranges.
  Range,
  /// A searcher for geospatial queries.
  Geospatial,
  /// A placeholder for custom, user-defined searchers.
  Custom,
}
