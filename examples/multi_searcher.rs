//! Multi-searcher example combining semantic, tag, and fuzzy search.

use searus::prelude::*;
use searus::searchers::{FuzzySearch, SemanticSearch, TaggedSearch};
use serde::{Deserialize, Serialize};

/// A blog post for demonstration.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Post {
  pub id: String,
  pub title: String,
  pub content: String,
  pub author: String,
  pub tags: Vec<String>,
  pub views: u32,
}

impl Post {
  /// Create a new post.
  pub fn new(
    id: impl Into<String>,
    title: impl Into<String>,
    content: impl Into<String>,
    author: impl Into<String>,
    tags: Vec<String>,
    views: u32,
  ) -> Self {
    Self {
      id: id.into(),
      title: title.into(),
      content: content.into(),
      author: author.into(),
      tags,
      views,
    }
  }
}

/// Create sample blog posts for examples.
pub fn sample_posts() -> Vec<Post> {
  vec![
        Post::new(
            "1",
            "Getting Started with Rust",
            "Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.",
            "Alice",
            vec!["rust".to_string(), "programming".to_string(), "tutorial".to_string()],
            1500,
        ),
        Post::new(
            "2",
            "Building a Search Engine",
            "Learn how to build a powerful search engine using Rust. We'll cover indexing, ranking, and query processing.",
            "Bob",
            vec!["rust".to_string(), "search".to_string(), "tutorial".to_string()],
            2300,
        ),
        Post::new(
            "3",
            "Advanced Rust Patterns",
            "Explore advanced design patterns in Rust including the builder pattern, type state pattern, and more.",
            "Alice",
            vec!["rust".to_string(), "advanced".to_string(), "patterns".to_string()],
            890,
        ),
        Post::new(
            "4",
            "Introduction to Machine Learning",
            "Machine learning basics: supervised learning, unsupervised learning, and neural networks explained.",
            "Charlie",
            vec!["ml".to_string(), "ai".to_string(), "tutorial".to_string()],
            3200,
        ),
        Post::new(
            "5",
            "Rust for Web Development",
            "Building web applications with Rust using frameworks like Actix-web and Rocket.",
            "Bob",
            vec!["rust".to_string(), "web".to_string(), "backend".to_string()],
            1750,
        ),
    ]
}

fn main() {
  println!("=== Searus Multi-Searcher Example ===\n");

  let posts = sample_posts();
  println!("Indexed {} blog posts\n", posts.len());

  // Configure semantic rules
  let semantic_rules = SemanticRules::builder()
    .field("title", FieldRule::bm25().priority(2))
    .field("content", FieldRule::tokenized().priority(1))
    .build();

  // Create searchers
  let semantic_searcher = SemanticSearch::new(semantic_rules);
  let tag_searcher = TaggedSearch::new();
  let fuzzy_searcher =
    FuzzySearch::new(vec!["title".to_string(), "content".to_string()]).with_threshold(0.75);

  // Build engine with multiple searchers and custom weights
  let engine = SearusEngine::builder()
    .with(Box::new(semantic_searcher))
    .with(Box::new(tag_searcher))
    .with(Box::new(fuzzy_searcher))
    .build();

  // Query with both text and tags
  println!("Query: text=\"rust\" + tags=[\"tutorial\"]\n");

  let query = Query::builder()
    .text("rust")
    .tags(vec!["tutorial".to_string()])
    .options(
      SearchOptions::default()
        .limit(5)
        .weight(SearcherKind::Semantic, 0.6)
        .weight(SearcherKind::Tags, 0.4),
    )
    .build();

  let results = engine.search(&posts, &query);

  for (i, result) in results.iter().enumerate() {
    println!(
      "{}. {} (score: {:.3})",
      i + 1,
      result.item.title,
      result.score
    );
    println!(
      "   Author: {} | Tags: {}",
      result.item.author,
      result.item.tags.join(", ")
    );

    // Show which searchers contributed
    for detail in &result.details {
      match detail {
        SearchDetail::Semantic { matched_terms, .. } => {
          println!("   ✓ Semantic: matched {}", matched_terms.join(", "));
        }
        SearchDetail::Tag { matched_tags, .. } => {
          println!("   ✓ Tags: matched {}", matched_tags.join(", "));
        }
        SearchDetail::Fuzzy {
          matched_term,
          original_term,
          similarity,
        } => {
          println!(
            "   ✓ Fuzzy: {} → {} (similarity: {:.2})",
            original_term, matched_term, similarity
          );
        }
        _ => {}
      }
    }
    println!();
  }
}
