//! Example demonstrating Tag Relationship Tree (TRT) usage with TaggedSearch.
//!
//! This example shows how to use TRT to expand tag queries hierarchically,
//! allowing related tags to contribute to search results with weighted scoring.

use searus::prelude::*;
use searus::searchers::tagged::{TagNode, TagRelationshipTree, TaggedSearch};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Post {
  id: u32,
  title: String,
  tags: Vec<String>,
}

fn main() {
  println!("=== Tag Relationship Tree (TRT) Example ===\n");

  // Create sample posts with various tags
  let posts = vec![
    Post {
      id: 1,
      title: "Introduction to Artificial Intelligence".to_string(),
      tags: vec!["ai".to_string()],
    },
    Post {
      id: 2,
      title: "Machine Learning Fundamentals".to_string(),
      tags: vec!["machine learning".to_string()],
    },
    Post {
      id: 3,
      title: "Python for Data Science".to_string(),
      tags: vec!["python".to_string()],
    },
    Post {
      id: 4,
      title: "Programming Best Practices".to_string(),
      tags: vec!["programming".to_string()],
    },
    Post {
      id: 5,
      title: "Deep Learning with PyTorch".to_string(),
      tags: vec!["deep learning".to_string(), "python".to_string()],
    },
    Post {
      id: 6,
      title: "Natural Language Processing".to_string(),
      tags: vec!["nlp".to_string(), "ai".to_string()],
    },
  ];

  // Define Tag Relationship Tree
  // Relationships: ai <-> machine learning -> python -> programming
  //                ai <-> deep learning -> python
  //                ai <-> nlp
  let trt_nodes = vec![
    TagNode {
      tag: "ai".to_string(),
      relationships: {
        let mut map = HashMap::new();
        map.insert("machine learning".to_string(), 0.7);
        map.insert("deep learning".to_string(), 0.8);
        map.insert("nlp".to_string(), 0.6);
        map
      },
    },
    TagNode {
      tag: "machine learning".to_string(),
      relationships: {
        let mut map = HashMap::new();
        map.insert("ai".to_string(), 0.9);
        map.insert("python".to_string(), 0.4);
        map.insert("deep learning".to_string(), 0.7);
        map
      },
    },
    TagNode {
      tag: "deep learning".to_string(),
      relationships: {
        let mut map = HashMap::new();
        map.insert("ai".to_string(), 0.8);
        map.insert("machine learning".to_string(), 0.7);
        map.insert("python".to_string(), 0.5);
        map
      },
    },
    TagNode {
      tag: "python".to_string(),
      relationships: {
        let mut map = HashMap::new();
        map.insert("programming".to_string(), 0.6);
        map.insert("machine learning".to_string(), 0.3);
        map
      },
    },
    TagNode {
      tag: "nlp".to_string(),
      relationships: {
        let mut map = HashMap::new();
        map.insert("ai".to_string(), 0.6);
        map
      },
    },
  ];

  let trt = TagRelationshipTree::new(trt_nodes);

  // Create TaggedSearch with TRT
  let tagged_search = TaggedSearch::new().with_trt(trt);
  let engine = SearusEngine::builder()
    .with(Box::new(tagged_search))
    .build();

  println!("TRT Structure:");
  println!("  ai (0.7)-> machine learning (0.4)-> python (0.6)-> programming");
  println!("  ai (0.8)-> deep learning (0.5)-> python");
  println!("  ai (0.6)-> nlp");
  println!("  machine learning (0.7)-> deep learning");
  println!();

  // Test 1: Query with no TRT expansion (depth = 0)
  println!("--- Test 1: Query 'ai' with NO TRT expansion (depth = 0) ---");
  let query = Query::builder()
    .tags(vec!["ai".to_string()])
    .filters(
      Query::filter(Query::COMPARE)
        .contains("title", "natural")
        .build(),
    )
    .build();
  let results = engine.search(&posts, &query);
  println!("Found {} results:", results.len());
  for (i, result) in results.iter().enumerate() {
    println!(
      "  {}. [Score: {:.3}] ID {}: {} (tags: {:?})",
      i + 1,
      result.score,
      result.item.id,
      result.item.title,
      result.item.tags
    );
  }
  println!();

  // Test 2: Query with TRT expansion depth = 1
  println!("--- Test 2: Query 'ai' with TRT expansion (depth = 1) ---");
  let query = Query::builder()
    .tags(vec!["ai".to_string()])
    .with_trt(1)
    .build();
  let results = engine.search(&posts, &query);
  println!("Found {} results:", results.len());
  println!(
    "Expected expanded tags: ai (1.0), machine learning (0.7), deep learning (0.8), nlp (0.6)"
  );
  for (i, result) in results.iter().enumerate() {
    println!(
      "  {}. [Score: {:.3}] ID {}: {} (tags: {:?})",
      i + 1,
      result.score,
      result.item.id,
      result.item.title,
      result.item.tags
    );
  }
  println!();

  // Test 3: Query with TRT expansion depth = 2
  println!("--- Test 3: Query 'ai' with TRT expansion (depth = 2) ---");
  let query = Query::builder()
    .tags(vec!["ai".to_string()])
    .with_trt(2)
    .build();
  let results = engine.search(&posts, &query);
  println!("Found {} results:", results.len());
  println!("Expected expanded tags:");
  println!("  - ai (1.0)");
  println!("  - machine learning (0.7), deep learning (0.8), nlp (0.6)");
  println!("  - python (0.7*0.4=0.28 or 0.8*0.5=0.4, max=0.4)");
  for (i, result) in results.iter().enumerate() {
    println!(
      "  {}. [Score: {:.3}] ID {}: {} (tags: {:?})",
      i + 1,
      result.score,
      result.item.id,
      result.item.title,
      result.item.tags
    );
  }
  println!();

  // Test 4: Query with TRT expansion depth = 3
  println!("--- Test 4: Query 'ai' with TRT expansion (depth = 3) ---");
  let query = Query::builder()
    .tags(vec!["ai".to_string()])
    .with_trt(3)
    .build();
  let results = engine.search(&posts, &query);
  println!("Found {} results:", results.len());
  println!("Expected: All posts should match, including 'programming' at depth 3");
  println!("  - programming strength: 0.4 * 0.6 = 0.24");
  for (i, result) in results.iter().enumerate() {
    println!(
      "  {}. [Score: {:.3}] ID {}: {} (tags: {:?})",
      i + 1,
      result.score,
      result.item.id,
      result.item.title,
      result.item.tags
    );
  }
  println!();

  // Test 5: Multiple query tags
  println!("--- Test 5: Query with multiple tags ['ai', 'python'] and depth = 1 ---");
  let query = Query::builder()
    .tags(vec!["ai".to_string(), "python".to_string()])
    .with_trt(1)
    .build();
  let results = engine.search(&posts, &query);
  println!("Found {} results:", results.len());
  for (i, result) in results.iter().enumerate() {
    println!(
      "  {}. [Score: {:.3}] ID {}: {} (tags: {:?})",
      i + 1,
      result.score,
      result.item.id,
      result.item.title,
      result.item.tags
    );
  }
  println!();

  // Test 6: Verify cycle handling
  println!("--- Test 6: Verifying cycle handling with large depth ---");
  let query = Query::builder()
    .tags(vec!["ai".to_string()])
    .with_trt(10) // Large depth to ensure cycles don't cause infinite loops
    .build();
  let results = engine.search(&posts, &query);
  println!(
    "Found {} results (should complete without hanging)",
    results.len()
  );
  println!("✓ Cycle detection working correctly");
  println!();

  println!("=== TRT Example Complete ===");
}
