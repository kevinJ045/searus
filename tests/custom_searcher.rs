use searus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Product {
  id: usize,
  name: String,
  category: String,
  price: f64,
}



struct PriceSearcher {
  max_price: f64,
}

impl PriceSearcher {
  fn new(max_price: f64) -> Self {
    Self { max_price }
  }
}

impl Searcher<Product> for PriceSearcher {
  fn kind(&self) -> SearcherKind {
    SearcherKind::Custom
  }

  fn search(&self, context: &SearchContext<Product>, _query: &Query) -> Vec<SearusMatch<Product>> {
    let mut matches = Vec::new();
    for (index, item) in context.items.iter().enumerate() {
      if item.price <= self.max_price {
        // Simple score: closer to 0 is better, but let's just use 1.0 for now
        // or maybe normalize by max_price: 1.0 - (price / max_price)
        let score = 1.0 - (item.price / self.max_price) as f32;
        matches.push(SearusMatch::new(item.clone(), score, index));
      }
    }
    matches
  }
}

#[test]
fn test_custom_searcher() {
  let products = vec![
    Product {
      id: 1,
      name: "Laptop".to_string(),
      category: "Electronics".to_string(),
      price: 1000.0,
    },
    Product {
      id: 2,
      name: "Phone".to_string(),
      category: "Electronics".to_string(),
      price: 500.0,
    },
    Product {
      id: 3,
      name: "Mouse".to_string(),
      category: "Electronics".to_string(),
      price: 20.0,
    },
  ];

  let engine = SearusEngine::builder()
    .with(Box::new(PriceSearcher::new(600.0)))
    .build();

  let query = Query::default();
  let results = engine.search(&products, &query);

  assert_eq!(results.len(), 2);
  assert_eq!(results[0].item.name, "Mouse"); // Lower price -> higher score
  assert_eq!(results[1].item.name, "Phone");
}
