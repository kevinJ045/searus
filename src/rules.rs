//! A domain-specific language (DSL) for configuring semantic text search.
//!
//! This module provides a set of structures that allow you to define detailed
//! rules for how the `SemanticSearch` searcher should operate. You can specify
//! which fields to search, what matching strategy to use for each field, and how
//! to weight the importance of different fields.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A container for all the semantic rules for a text-based search.
///
/// `SemanticRules` acts as the top-level configuration for the `SemanticSearch`
/// searcher. It holds rules for both top-level fields and nested objects.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SemanticRules {
  /// A map of field names to the `FieldRule` that should be applied to them.
  pub fields: HashMap<String, FieldRule>,
  /// A map of nested object names to the `ObjectRule` that should be applied.
  pub objects: HashMap<String, ObjectRule>,
}

impl SemanticRules {
  /// Creates a new `SemanticRulesBuilder` to construct the rules fluently.
  pub fn builder() -> SemanticRulesBuilder {
    SemanticRulesBuilder::default()
  }
}

/// A builder for creating `SemanticRules` instances.
#[derive(Debug, Default)]
pub struct SemanticRulesBuilder {
  fields: HashMap<String, FieldRule>,
  objects: HashMap<String, ObjectRule>,
}

impl SemanticRulesBuilder {
  /// Adds a rule for a specific field.
  pub fn field(mut self, name: impl Into<String>, rule: FieldRule) -> Self {
    self.fields.insert(name.into(), rule);
    self
  }

  /// Adds a rule for a nested object.
  pub fn object(mut self, name: impl Into<String>, rule: ObjectRule) -> Self {
    self.objects.insert(name.into(), rule);
    self
  }

  /// Builds the final `SemanticRules` object.
  pub fn build(self) -> SemanticRules {
    SemanticRules {
      fields: self.fields,
      objects: self.objects,
    }
  }
}

/// Defines the search behavior for a single field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldRule {
  /// The matching strategy to use for this field.
  pub matcher: Matcher,
  /// The priority of this field relative to others. A higher priority gives
  /// the field more influence on the final score.
  #[serde(default = "default_priority")]
  pub priority: u32,
  /// A multiplier applied to the score of this field, allowing you to "boost"
  /// its importance.
  #[serde(default = "default_boost")]
  pub boost: f32,
}

/// Returns the default priority for a field (1).
fn default_priority() -> u32 {
  1
}

/// Returns the default boost for a field (1.0).
fn default_boost() -> f32 {
  1.0
}

impl Default for FieldRule {
  /// Creates a default `FieldRule` with the `Tokenized` matcher.
  fn default() -> Self {
    Self {
      matcher: Matcher::Tokenized,
      priority: default_priority(),
      boost: default_boost(),
    }
  }
}

impl FieldRule {
  /// Creates a new `FieldRule` with a specified matcher and default priority and boost.
  pub fn new(matcher: Matcher) -> Self {
    Self {
      matcher,
      ..Default::default()
    }
  }

  /// Creates a `FieldRule` for exact, case-insensitive matching.
  pub fn exact() -> Self {
    Self::new(Matcher::Exact)
  }

  /// Creates a `FieldRule` for relevance scoring using the BM25 algorithm.
  pub fn bm25() -> Self {
    Self::new(Matcher::BM25)
  }

  /// Creates a `FieldRule` for simple token-based matching.
  pub fn tokenized() -> Self {
    Self::new(Matcher::Tokenized)
  }

  /// Creates a `FieldRule` for fuzzy (approximate) matching.
  pub fn fuzzy() -> Self {
    Self::new(Matcher::Fuzzy)
  }

  /// Sets the priority for this field rule.
  pub fn priority(mut self, priority: u32) -> Self {
    self.priority = priority;
    self
  }

  /// Sets the boost factor for this field rule.
  pub fn boost(mut self, boost: f32) -> Self {
    self.boost = boost;
    self
  }
}

/// Defines the matching strategy to be used for a field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Matcher {
  /// Requires an exact, case-insensitive match of the query within the field's text.
  Exact,
  /// Uses the BM25 algorithm to score the relevance of the field based on term
  /// frequency and inverse document frequency.
  BM25,
  /// A simple strategy that scores based on the frequency of query tokens in the field.
  Tokenized,
  /// Uses a fuzzy matching algorithm (like Jaro-Winkler) to find approximate matches.
  /// Note: This is typically handled by the `FuzzySearch` searcher.
  Fuzzy,
}

/// Defines the search behavior for a nested object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectRule {
  /// The method used to access the nested object.
  pub access: ObjectAccess,
  /// A map of field names within the nested object to their corresponding `FieldRule`.
  pub fields: HashMap<String, FieldRule>,
}

impl ObjectRule {
  /// Creates a builder for an `ObjectRule` that is accessed directly by its field name.
  pub fn direct() -> ObjectRuleBuilder {
    ObjectRuleBuilder {
      access: ObjectAccess::Direct,
      fields: HashMap::new(),
    }
  }
}

/// A builder for creating `ObjectRule` instances.
#[derive(Debug)]
pub struct ObjectRuleBuilder {
  access: ObjectAccess,
  fields: HashMap<String, FieldRule>,
}

impl ObjectRuleBuilder {
  /// Adds a rule for a specific field within the nested object.
  pub fn field(mut self, name: impl Into<String>, rule: FieldRule) -> Self {
    self.fields.insert(name.into(), rule);
    self
  }

  /// Builds the final `ObjectRule` object.
  pub fn build(self) -> ObjectRule {
    ObjectRule {
      access: self.access,
      fields: self.fields,
    }
  }
}

/// Defines how a nested object is accessed within a parent object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectAccess {
  /// The object is a direct property of its parent.
  Direct,
}
