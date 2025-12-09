//! Basic semantic search example.

use searus_core::prelude::*;
use searus_searchers::SemanticSearch;
use searus_examples::{sample_posts, Post};

fn main() {
    println!("=== Searus Semantic Search Example ===\n");

    // Create sample blog posts
    let posts = sample_posts();
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

    // Example queries
    let queries = vec![
        "rust programming",
        "search engine",
        "machine learning",
        "web development",
    ];

    for query_text in queries {
        println!("Query: \"{}\"\n", query_text);

        let query = Query::builder()
            .text(query_text)
            .options(SearchOptions::default().limit(3))
            .build();

        let results = engine.search(&posts, &query);

        if results.is_empty() {
            println!("  No results found.\n");
        } else {
            for (i, result) in results.iter().enumerate() {
                println!("  {}. {} (score: {:.3})", i + 1, result.item.title, result.score);
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
        println!("---\n");
    }
}
