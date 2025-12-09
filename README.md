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
searus = "0.1.0"
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
    let engine = SearusEngine::builder()
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
)?;

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
```

## Architecture

```
searus/
├── types.rs          # Core types (Query, SearusMatch, SearchOptions)
├── searcher.rs       # Searcher trait
├── engine.rs         # SearusEngine (orchestrates multiple searchers)
├── rules.rs          # Semantic rules DSL
├── filter.rs         # Filter expressions (future)
├── embeddings/       # Embedding provider traits
│   └── mod.rs
├── index/            # Storage adapters
│   ├── adapter.rs    # IndexAdapter trait
│   └── memory.rs     # In-memory implementation
└── searchers/        # Search implementations
    ├── tokenizer.rs  # Text tokenization
    ├── bm25.rs       # BM25 scorer
    ├── semantic.rs   # Semantic search
    ├── tagged.rs     # Tag search
    └── fuzzy.rs      # Fuzzy search
```

## Roadmap

- [ ] Filter expressions (range queries, boolean logic)
- [ ] Geospatial search
- [ ] Image search with embeddings
- [ ] Persistent index adapters (PostgreSQL, Redis)
- [ ] Query DSL improvements
- [ ] Performance benchmarks
- [ ] More tokenization strategies

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- BM25 implementation inspired by search engine research
- Fuzzy matching powered by the excellent [strsim](https://crates.io/crates/strsim) crate
- Text tokenization using [unicode-segmentation](https://crates.io/crates/unicode-segmentation)
