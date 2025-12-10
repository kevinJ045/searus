//! Basic semantic search example.
use searus::prelude::*;
use searus::searchers::SemanticSearch;

#[path = "common/mod.rs"]
mod common;

fn main() {
  println!("=== Searus Search Time Check Example ===\n");

  println!("Features: {:?}", std::env::var("CARGO_FEATURES"));

  // Create sample blog posts
  let mut posts = common::sample_posts();
  let mut i = 1;

  let start = std::time::Instant::now();

  let title = "Filler Post".to_string();
  let content = "A very long content of a filler post.".to_string();
  let username = "SomeFillingMan".to_string();
  let tags = vec!["filler".to_string(), "post".to_string(), "demo".to_string()];

  while i < 100_000 {
    posts.push(common::Post::new(
      format!("{}", i + 5),
      title.clone(),
      content.clone(),
      username.clone(),
      tags.clone(),
      1500,
    ));
    i += 1;
  }

  let elapsed_filling = start.elapsed();

  println!("Indexed {} blog posts\n", posts.len());

  // Configure semantic rules
  let rules = SemanticRules::builder()
    .field("title", FieldRule::bm25().priority(3).boost(2.0))
    .field("content", FieldRule::bm25().priority(2).boost(1.0))
    .field("author", FieldRule::exact().priority(1).boost(1.5))
    .build();

  // Create semantic searcher
  let semantic_searcher = SemanticSearch::new(rules);

  // Build search engine
  let engine = SearusEngine::builder()
    .with(Box::new(semantic_searcher))
    .build();

  let query = Query::builder()
    .text("search")
    .options(SearchOptions::default().limit(5))
    .build();

  let results = engine.search(&posts, &query);

  if results.is_empty() {
    println!("  No results found.\n");
  } else {
    println!("Found {} posts\n", results.len());
    for (i, result) in results.iter().enumerate() {
      println!(
        "  {}. {} (score: {:.3})",
        i + 1,
        result.item.title,
        result.score
      );
      println!("     Author: {}", result.item.author);

      if !result.field_scores.is_empty() {
        print!("     Field scores: ");
        for (field, score) in &result.field_scores {
          print!("{}={:.2} ", field, score);
        }
        println!();
      }

      if !result.details.is_empty() {
        for detail in &result.details {
          match detail {
            SearchDetail::Semantic { matched_terms, .. } => {
              println!("     Matched terms: {}", matched_terms.join(", "));
            }
            _ => {}
          }
        }
      }
      println!();
    }
  }

  println!("Filling entities took {:?}", elapsed_filling);
  println!("Results shown in {:?}", start.elapsed());
}
