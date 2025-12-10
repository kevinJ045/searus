//! Verification example for filters.
use searus::prelude::*;
use searus::searchers::{FuzzySearch, SemanticSearch, TaggedSearch};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Product {
  id: u32,
  name: String,
  category: String,
  price: f64,
  tags: Vec<String>,
  description: String,
}

impl Product {
  fn new(id: u32, name: &str, category: &str, price: f64, tags: &[&str], description: &str) -> Self {
    Self {
      id,
      name: name.to_string(),
      category: category.to_string(),
      price,
      tags: tags.iter().map(|s| s.to_string()).collect(),
      description: description.to_string(),
    }
  }
}

fn main() {
  println!("=== Searus Filter Verification ===\n");

  let products = vec![
    Product::new(1, "Laptop Pro", "Electronics", 1200.0, &["computer", "work"], "High performance laptop"),
    Product::new(2, "Smartphone X", "Electronics", 800.0, &["mobile", "5g"], "Latest smartphone"),
    Product::new(3, "Running Shoes", "Sports", 120.0, &["shoes", "fitness"], "Comfortable running shoes"),
    Product::new(4, "Coffee Maker", "Home", 50.0, &["kitchen", "coffee"], "Automatic coffee maker"),
    Product::new(5, "Gaming Mouse", "Electronics", 60.0, &["computer", "gaming"], "RGB gaming mouse for computer"),
  ];

  // 1. Semantic Search with Filter
  println!("--- Semantic Search (query: 'computer', filter: price < 100) ---");
  let semantic_rules = SemanticRules::builder()
    .field("name", FieldRule::bm25())
    .field("description", FieldRule::bm25())
    .build();
  let semantic_searcher = SemanticSearch::new(semantic_rules);
  
  let engine = SearusEngine::builder()
    .with(Box::new(semantic_searcher))
    .build();

  let query = Query::builder()
    .text("computer")
    .filters(Query::filter(Query::COMPARE).lt("price", 100.0).build())
    .build();

  let results = engine.search(&products, &query);
  for match_item in &results {
    println!("Found: {} (${})", match_item.item.name, match_item.item.price);
  }
  assert_eq!(results.len(), 1);
  assert_eq!(results[0].item.name, "Gaming Mouse");
  println!("Semantic Check: PASSED\n");


  // 2. Fuzzy Search with Filter
  println!("--- Fuzzy Search (query: 'laptap', filter: category == 'Electronics') ---");
  let fuzzy_searcher = FuzzySearch::new(vec!["name".to_string()]);
  let engine = SearusEngine::builder()
    .with(Box::new(fuzzy_searcher))
    .build();

  let query = Query::builder()
    .text("laptap") // Typo intended
    .filters(Query::filter(Query::COMPARE).eq("category", "Electronics").build())
    .build();

  let results = engine.search(&products, &query);
  for match_item in &results {
    println!("Found: {} ({})", match_item.item.name, match_item.item.category);
  }
  assert_eq!(results.len(), 1);
  assert_eq!(results[0].item.name, "Laptop Pro");
  println!("Fuzzy Check: PASSED\n");

  // 3. Tagged Search with Filter
  println!("--- Tagged Search (tags: ['computer'], filter: price > 1000) ---");
  let tagged_searcher = TaggedSearch::new();
  let engine = SearusEngine::builder()
    .with(Box::new(tagged_searcher))
    .build();

  let query = Query::builder()
    .tags(vec!["computer".to_string()])
    .filters(Query::filter(Query::COMPARE).gt("price", 1000.0).build())
    .build();

  let results = engine.search(&products, &query);
  for match_item in &results {
    println!("Found: {} (${})", match_item.item.name, match_item.item.price);
  }
  assert_eq!(results.len(), 1);
  assert_eq!(results[0].item.name, "Laptop Pro");
  println!("Tagged Check: PASSED\n");
  
  println!("All checks passed!");
}
