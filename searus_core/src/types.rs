//! Core data types for the Searus search engine.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type alias for entity identifiers.
pub type EntityId = String;

/// A search match result containing the matched item, score, and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearusMatch<T> {
    /// The matched item or entity.
    pub item: T,
    /// The final normalized score (0.0 to 1.0, higher is better).
    pub score: f32,
    /// Optional per-field score breakdown for explainability.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub field_scores: HashMap<String, f32>,
    /// Searcher-specific metadata about this match.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<SearchDetail>,
}

impl<T> SearusMatch<T> {
    /// Create a new search match with the given item and score.
    pub fn new(item: T, score: f32) -> Self {
        Self {
            item,
            score,
            field_scores: HashMap::new(),
            details: Vec::new(),
        }
    }

    /// Add a field score to this match.
    pub fn with_field_score(mut self, field: impl Into<String>, score: f32) -> Self {
        self.field_scores.insert(field.into(), score);
        self
    }

    /// Add a search detail to this match.
    pub fn with_detail(mut self, detail: SearchDetail) -> Self {
        self.details.push(detail);
        self
    }
}

/// Searcher-specific metadata about a match.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SearchDetail {
    /// Semantic text matching detail.
    Semantic {
        matched_terms: Vec<String>,
        field: String,
        weight: f32,
    },
    /// Vector similarity detail.
    Vector {
        distance: f32,
        similarity: f32,
    },
    /// Tag matching detail.
    Tag {
        matched_tags: Vec<String>,
        total_tags: usize,
    },
    /// Fuzzy matching detail.
    Fuzzy {
        matched_term: String,
        original_term: String,
        similarity: f32,
    },
    /// Image matching detail.
    Image {
        similarity: f32,
    },
}

/// A search query with multiple optional search modes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Query {
    /// Text query for semantic search.
    pub text: Option<String>,
    /// Pre-computed vector for vector search.
    pub vector: Option<Vec<f32>>,
    /// Tags to match.
    pub tags: Option<Vec<String>>,
    /// Image data for image search.
    pub image: Option<ImageData>,
    /// Filter expressions to apply.
    pub filters: Option<crate::filter::FilterExpr>,
    /// Search options.
    #[serde(default)]
    pub options: SearchOptions,
}

impl Query {
    /// Create a new query builder.
    pub fn builder() -> QueryBuilder {
        QueryBuilder::default()
    }
}

/// Builder for constructing queries.
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
    /// Set the text query.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Set the vector query.
    pub fn vector(mut self, vector: Vec<f32>) -> Self {
        self.vector = Some(vector);
        self
    }

    /// Set the tags to match.
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Set the image data.
    pub fn image(mut self, image: ImageData) -> Self {
        self.image = Some(image);
        self
    }

    /// Set the filter expression.
    pub fn filters(mut self, filters: crate::filter::FilterExpr) -> Self {
        self.filters = Some(filters);
        self
    }

    /// Set the search options.
    pub fn options(mut self, options: SearchOptions) -> Self {
        self.options = options;
        self
    }

    /// Build the query.
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

/// Image data for image-based search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    /// Raw image bytes.
    pub bytes: Vec<u8>,
    /// MIME type (e.g., "image/png", "image/jpeg").
    pub mime_type: Option<String>,
    /// Image width in pixels.
    pub width: Option<u32>,
    /// Image height in pixels.
    pub height: Option<u32>,
}

/// Search options for pagination, weighting, and timeouts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    /// Number of results to skip (for pagination).
    #[serde(default)]
    pub skip: usize,
    /// Maximum number of results to return.
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Timeout in milliseconds (0 = no timeout).
    #[serde(default)]
    pub timeout_ms: u64,
    /// Per-searcher weights for score blending.
    #[serde(default)]
    pub weights: HashMap<SearcherKind, f32>,
}

fn default_limit() -> usize {
    20
}

impl Default for SearchOptions {
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
    /// Set the skip value.
    pub fn skip(mut self, skip: usize) -> Self {
        self.skip = skip;
        self
    }

    /// Set the limit value.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Set the timeout in milliseconds.
    pub fn timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Set a weight for a specific searcher kind.
    pub fn weight(mut self, kind: SearcherKind, weight: f32) -> Self {
        self.weights.insert(kind, weight);
        self
    }
}

/// Identifies the type of searcher.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SearcherKind {
    Semantic,
    Vector,
    Tags,
    Image,
    Fuzzy,
    Range,
    Geospatial,
    Custom,
}
