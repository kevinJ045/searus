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
