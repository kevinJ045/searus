//! Multi-searcher example combining semantic, tag, and fuzzy search.

use searus_core::prelude::*;
use searus_searchers::{SemanticSearch, TaggedSearch, FuzzySearch};
use searus_examples::{sample_posts, Post};

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
    let fuzzy_searcher = FuzzySearch::new(vec!["title".to_string(), "content".to_string()])
        .with_threshold(0.75);

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
                .weight(SearcherKind::Tags, 0.4)
        )
        .build();

    let results = engine.search(&posts, &query);

    for (i, result) in results.iter().enumerate() {
        println!("{}. {} (score: {:.3})", i + 1, result.item.title, result.score);
        println!("   Author: {} | Tags: {}", result.item.author, result.item.tags.join(", "));
        
        // Show which searchers contributed
        for detail in &result.details {
            match detail {
                SearchDetail::Semantic { matched_terms, .. } => {
                    println!("   ✓ Semantic: matched {}", matched_terms.join(", "));
                }
                SearchDetail::Tag { matched_tags, .. } => {
                    println!("   ✓ Tags: matched {}", matched_tags.join(", "));
                }
                SearchDetail::Fuzzy { matched_term, original_term, similarity } => {
                    println!("   ✓ Fuzzy: {} → {} (similarity: {:.2})", original_term, matched_term, similarity);
                }
                _ => {}
            }
        }
        println!();
    }
}
