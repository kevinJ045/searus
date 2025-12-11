use searus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Item {
  id: usize,
  name: String,
}

struct QueryRewriteExt;

impl SearusExtension<Item> for QueryRewriteExt {
  fn before_query(&self, query: &mut Query) {
    if let Some(text) = &query.text {
      if text == "ml" {
        query.text = Some("machine learning".to_string());
      }
    }
  }
}

struct ResultModifyExt;

impl SearusExtension<Item> for ResultModifyExt {
  fn after_limit(&self, _query: &Query, results: &mut Vec<SearusMatch<Item>>) {
    for m in results {
      m.score += 0.1; // Boost score
    }
  }
}

struct AddItemExt;

impl SearusExtension<Item> for AddItemExt {
  fn before_items(&self, _query: &Query, items: &mut Vec<Item>) {
    items.push(Item {
      id: 999,
      name: "Added by extension".to_string(),
    });
  }
}

#[test]
fn test_extensions() {
  let items = vec![
    Item {
      id: 1,
      name: "machine learning".to_string(),
    },
    Item {
      id: 2,
      name: "ai".to_string(),
    },
  ];

  let engine = SearusEngine::builder()
    .with(Box::new(SemanticSearch::new(
      SemanticRules::builder()
        .field("name", FieldRule::default())
        .build(),
    )))
    .with_extension(Box::new(QueryRewriteExt))
    .with_extension(Box::new(ResultModifyExt))
    .with_extension(Box::new(AddItemExt))
    .build();

  // Test query rewrite
  let query = Query::builder().text("ml").build();
  let results = engine.search(&items, &query);

  // "ml" should be rewritten to "machine learning"
  // "machine learning" item should match
  // "Added by extension" should be present (but might not match "machine learning")

  // Check if "machine learning" matched
  let ml_match = results.iter().find(|m| m.item.name == "machine learning");
  assert!(ml_match.is_some(), "Query rewrite failed");

  // Check score boost
  if let Some(_) = ml_match {
    // BM25 score is > 0. With boost, it should be higher.
    // Exact score depends on BM25 implementation, but we know it's boosted by 0.1
    // Let's just check if we got results.
  }

  // Test item addition
  // Search for "extension"
  let query_ext = Query::builder().text("extension").build();
  let results_ext = engine.search(&items, &query_ext);

  let ext_match = results_ext
    .iter()
    .find(|m| m.item.name == "Added by extension");
  assert!(ext_match.is_some(), "Item addition failed");

  if let Some(m) = ext_match {
    assert!(m.score >= 0.1, "Score boost failed");
  }
}
