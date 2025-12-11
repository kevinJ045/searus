# Searus

A flexible, multi-modal search engine library for Rust.

[![Crates.io](https://img.shields.io/crates/v/searus.svg)](https://crates.io/crates/searus)
[![Documentation](https://docs.rs/searus/badge.svg)](https://docs.rs/searus)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

Searus is a powerful search engine library that provides multiple search strategies out of the box:

- **Semantic Search** - BM25-based text search with configurable field rules
- **Tag-based Search** - Exact and fuzzy tag matching
- **Fuzzy Search** - String similarity matching using Jaro-Winkler distance
- **Vector Search** - Nearest neighbor search with embeddings (via index adapters)
- **Multi-modal Search** - Combine multiple search strategies with weighted scoring

## Features

- 🚀 **Fast and Lightweight** - Zero-cost abstractions with minimal dependencies
- 🔧 **Flexible Configuration** - Fine-tune search behavior with semantic rules
- 🎯 **Multi-Strategy** - Combine different search methods with custom weights
- 📊 **Score Transparency** - Detailed per-field scores and match explanations
- 🔌 **Pluggable Storage** - Bring your own index with the `IndexAdapter` trait
- 🎨 **Type-Safe** - Generic over your document types with `serde` support

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
searus = "0.0.3"
```

## Quick Start

```rust
use searus::prelude::*;
use searus::searchers::SemanticSearch;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Post {
    title: String,
    content: String,
    tags: Vec<String>,
}

fn main() {
    // Configure semantic search rules
    let rules = SemanticRules::builder()
        .field("title", FieldRule::bm25().priority(3).boost(2.0))
        .field("content", FieldRule::bm25().priority(2).boost(1.0))
        .build();

    // Create a searcher
    let searcher = SemanticSearch::new(rules);

    // Build the search engine
    let engine: SearusEngine<Post> = SearusEngine::builder()
        .with(Box::new(searcher))
        .build();

    // Your documents
    let posts = vec![
        Post {
            title: "Getting Started with Rust".to_string(),
            content: "Rust is a systems programming language...".to_string(),
            tags: vec!["rust".to_string(), "tutorial".to_string()],
        },
        // ... more posts
    ];

    // Search!
    let query = Query::builder()
        .text("rust programming")
        .options(SearchOptions::default().limit(10))
        .build();

    let results = engine.search(&posts, &query);

    for result in results {
        println!("{} (score: {:.3})", result.item.title, result.score);
    }
}
```

## Search Strategies

### Semantic Search

BM25-based text search with configurable field rules and matching strategies:

```rust
use searus::prelude::*;
use searus::searchers::SemanticSearch;

let rules = SemanticRules::builder()
    .field("title", FieldRule::bm25().priority(3).boost(2.0))
    .field("content", FieldRule::tokenized().priority(1))
    .field("author", FieldRule::exact())
    .build();

let searcher = SemanticSearch::new(rules);
```

**Matching Strategies:**
- `Matcher::BM25` - Full BM25 scoring with IDF
- `Matcher::Tokenized` - Simple term frequency matching
- `Matcher::Exact` - Case-insensitive exact string matching
- `Matcher::Fuzzy` - Delegated to `FuzzySearch`

### Tag-based Search

Match documents by tags with configurable field names:

```rust
use searus::searchers::TaggedSearch;

// Default field name is "tags"
let tag_searcher = TaggedSearch::new();

// Or specify a custom field
let tag_searcher = TaggedSearch::with_field("categories");

let query = Query::builder()
    .tags(vec!["rust".to_string(), "tutorial".to_string()])
    .build();
```

### Tag Relationship Trees (TRT)

Enhance tag-based search by defining relationships between tags. This allows queries for a parent tag (e.g., "programming") to automatically include results for child tags (e.g., "rust", "python").

```rust
use searus::searchers::tagged::{TagNode, TagRelationshipTree};
use std::collections::HashMap;

// Define tag relationships
let nodes = vec![
    TagNode {
        tag: "rust".to_string(),
        relationships: HashMap::from([("programming".to_string(), 0.8)]),
    },
    TagNode {
        tag: "python".to_string(),
        relationships: HashMap::from([("programming".to_string(), 0.7)]),
    },
];

let trt = TagRelationshipTree::new(nodes);

// Configure searcher with TRT
let tag_searcher = TaggedSearch::new().with_trt(trt);

// Query with TRT expansion (depth 1)
let query = Query::builder()
    .tags(vec!["programming".to_string()])
    .with_trt(1) 
    .build();
```

### Fuzzy Search

String similarity matching using Jaro-Winkler distance:

```rust
use searus::searchers::FuzzySearch;

let fuzzy_searcher = FuzzySearch::new(vec!["title".to_string(), "content".to_string()])
    .with_threshold(0.8); // Minimum similarity: 0.0 to 1.0

let query = Query::builder()
    .text("programing") // Will match "programming"
    .build();
```

### Multi-Strategy Search

Combine multiple searchers with custom weights:

```rust
use searus::prelude::*;
use searus::searchers::{SemanticSearch, TaggedSearch, FuzzySearch};

let semantic_rules = SemanticRules::builder()
    .field("title", FieldRule::bm25().priority(2))
    .field("content", FieldRule::tokenized())
    .build();

let engine = SearusEngine::builder()
    .with(Box::new(SemanticSearch::new(semantic_rules)))
    .with(Box::new(TaggedSearch::new()))
    .with(Box::new(FuzzySearch::new(vec!["title".to_string()])))
    .build();

let query = Query::builder()
    .text("rust")
    .tags(vec!["tutorial".to_string()])
    .options(
        SearchOptions::default()
            .weight(SearcherKind::Semantic, 0.6)
            .weight(SearcherKind::Tags, 0.4)
    )
    .build();
```

## Extensions

Customize the search lifecycle with the `SearusExtension` trait. Extensions can intercept queries, modify items, and alter results.

```rust
use searus::prelude::*;

struct LoggingExtension;

impl<T: Searchable> SearusExtension<T> for LoggingExtension {
    fn before_query(&self, query: &mut Query) {
        println!("Executing query: {:?}", query);
    }

    fn after_searcher(&self, _query: &Query, results: &mut Vec<SearusMatch<T>>) {
        println!("Searcher returned {} results", results.len());
    }
}

// Register extension in the engine
let engine: SearusEngine<Post> = SearusEngine::builder()
    .with(Box::new(searcher))
    .with_extension(Box::new(LoggingExtension))
    .build();
```

## Custom Searchers

Implement your own search strategies by implementing the `Searcher` trait.

```rust
use searus::prelude::*;

struct MySearcher;

impl<T: Searchable> Searcher<T> for MySearcher {
    fn kind(&self) -> SearcherKind {
        SearcherKind::Custom
    }

    fn search(&self, context: &SearchContext<T>, query: &Query) -> Vec<SearusMatch<T>> {
        // Implement your search logic here
        vec![]
    }
}
```

This allows you to plug in any algorithm (e.g., TF-IDF, LSH, experimental models) and combine it with built-in searchers.

## Optimization

For large datasets (100k+ entities), consider these optimization strategies:

1.  **Precomputation**: Pre-tokenize text and pre-compute embeddings.
2.  **Parallelism**: Enable the `parallel` feature to use `rayon` for concurrent search execution.
3.  **Early Filtering**: Apply cheap filters (tags, exact matches) before expensive semantic or vector searches.
4.  **Approximate Nearest Neighbors (ANN)**: Use an `IndexAdapter` that supports ANN (e.g., HNSW) instead of brute-force KNN.

## Index Adapters

Searus supports pluggable storage backends through the `IndexAdapter` trait:

```rust
use searus::index::{IndexAdapter, InMemIndex};

// Built-in in-memory index
let mut index: InMemIndex<Post> = InMemIndex::new();

index.put(
    "post-1".to_string(),
    post,
    Some(embedding_vector), // Optional vector for KNN search
    Some(vec!["rust".to_string()]), // Optional tags
).unwrap();

// Find nearest neighbors
let neighbors = index.knn(&query_vector, 10);
```

Implement `IndexAdapter` for your own storage backend (e.g., PostgreSQL, Redis, Qdrant).

## Embeddings

Searus provides traits for embedding providers:

```rust
use searus::embeddings::{TextEmbedder, StubTextEmbedder};

// Built-in stub embedder for testing
let embedder = StubTextEmbedder::new(384); // 384-dimensional vectors

let embedding = embedder.embed("Hello, world!")?;

// Implement TextEmbedder for your own provider (OpenAI, Cohere, local models, etc.)
```

## Query Options

Fine-tune your search with query options:

```rust
let query = Query::builder()
    .text("rust programming")
    .tags(vec!["tutorial".to_string()])
    .options(
        SearchOptions::default()
            .limit(20)                              // Max results
            .skip(10)                               // Pagination offset
            .timeout_ms(5000)                       // Search timeout
            .weight(SearcherKind::Semantic, 0.7)    // Searcher weights
            .weight(SearcherKind::Tags, 0.3)
    )
    .filters(
        // views >= 1000 OR  author = Bob
        Query::filter(Query::OR)
            .with(
              Query::filter(Query::COMPARE)
                  .ge("views", 1000)
                  .build()
            )
            .with(
              Query::filter(Query::COMPARE)
                  .eq("author", "Bob")
                  .build()
            )
            .build()
    )
    .build();
```

## Score Transparency

Searus provides detailed scoring information:

```rust
for result in results {
    println!("Score: {:.3}", result.score);
    
    // Per-field scores
    for (field, score) in &result.field_scores {
        println!("  {}: {:.3}", field, score);
    }
    
    // Match details
    for detail in &result.details {
        match detail {
            SearchDetail::Semantic { matched_terms, .. } => {
                println!("  Matched: {}", matched_terms.join(", "));
            }
            SearchDetail::Tag { matched_tags, .. } => {
                println!("  Tags: {}", matched_tags.join(", "));
            }
            SearchDetail::Fuzzy { original_term, matched_term, similarity } => {
                println!("  {} → {} ({:.2})", original_term, matched_term, similarity);
            }
            _ => {}
        }
    }
}
```

## Examples

Run the included examples:

```bash
# Basic semantic search
cargo run --example basic_semantic

# Multi-strategy search
cargo run --example multi_searcher

# Time check
cargo run --example time_check --features parallel

# Filters example
cargo run --example verify_filters

# Tagged TRT search
cargo run --example tagged_trt
```

## Roadmap

- [x] **Multithreaded Operations**: Run all search operations in parallel.
- [x] **Filter Expressions**: Range queries, boolean logic, and complex filtering.
- [ ] **Async Operations**: Asynchronous entity search logic.
- [ ] **Geospatial Search**: Location-based querying.
- [ ] **Image Search**: Image-to-image and text-to-image search using embeddings.
- [ ] **Persistent Storage**: Disk-backed index adapters (e.g., using `sled` or `rocksdb`).
- [ ] **Distributed Search**: Sharding and clustering for massive datasets.
- [ ] **Performance**: SIMD optimizations and advanced caching strategies.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- BM25 implementation inspired by search engine research
- Fuzzy matching powered by the excellent [strsim](https://crates.io/crates/strsim) crate
- Text tokenization using [unicode-segmentation](https://crates.io/crates/unicode-segmentation)
