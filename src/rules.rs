//! Semantic rules DSL for configuring text-based search.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Semantic rules configuration for text-based search.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SemanticRules {
    /// Field-level rules.
    pub fields: HashMap<String, FieldRule>,
    /// Nested object rules.
    pub objects: HashMap<String, ObjectRule>,
}

impl SemanticRules {
    /// Create a new rules builder.
    pub fn builder() -> SemanticRulesBuilder {
        SemanticRulesBuilder::default()
    }
}

/// Builder for semantic rules.
#[derive(Debug, Default)]
pub struct SemanticRulesBuilder {
    fields: HashMap<String, FieldRule>,
    objects: HashMap<String, ObjectRule>,
}

impl SemanticRulesBuilder {
    /// Add a field rule.
    pub fn field(mut self, name: impl Into<String>, rule: FieldRule) -> Self {
        self.fields.insert(name.into(), rule);
        self
    }

    /// Add an object rule.
    pub fn object(mut self, name: impl Into<String>, rule: ObjectRule) -> Self {
        self.objects.insert(name.into(), rule);
        self
    }

    /// Build the semantic rules.
    pub fn build(self) -> SemanticRules {
        SemanticRules {
            fields: self.fields,
            objects: self.objects,
        }
    }
}

/// Rule for a single field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldRule {
    /// Matching strategy.
    pub matcher: Matcher,
    /// Priority/weight for this field (higher = more important).
    #[serde(default = "default_priority")]
    pub priority: u32,
    /// Boost factor for scores from this field.
    #[serde(default = "default_boost")]
    pub boost: f32,
}

fn default_priority() -> u32 {
    1
}

fn default_boost() -> f32 {
    1.0
}

impl Default for FieldRule {
    fn default() -> Self {
        Self {
            matcher: Matcher::Tokenized,
            priority: default_priority(),
            boost: default_boost(),
        }
    }
}

impl FieldRule {
    /// Create a new field rule with the given matcher.
    pub fn new(matcher: Matcher) -> Self {
        Self {
            matcher,
            ..Default::default()
        }
    }

    /// Create an exact match rule.
    pub fn exact() -> Self {
        Self::new(Matcher::Exact)
    }

    /// Create a BM25 rule.
    pub fn bm25() -> Self {
        Self::new(Matcher::BM25)
    }

    /// Create a tokenized rule.
    pub fn tokenized() -> Self {
        Self::new(Matcher::Tokenized)
    }

    /// Create a fuzzy rule.
    pub fn fuzzy() -> Self {
        Self::new(Matcher::Fuzzy)
    }

    /// Set the priority.
    pub fn priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Set the boost factor.
    pub fn boost(mut self, boost: f32) -> Self {
        self.boost = boost;
        self
    }
}

/// Matching strategy for a field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Matcher {
    /// Exact string match (case-insensitive).
    Exact,
    /// BM25 scoring.
    BM25,
    /// Simple tokenized matching (term frequency).
    Tokenized,
    /// Fuzzy matching with edit distance.
    Fuzzy,
}

/// Rule for nested objects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectRule {
    /// How to access the object.
    pub access: ObjectAccess,
    /// Field rules within this object.
    pub fields: HashMap<String, FieldRule>,
}

impl ObjectRule {
    /// Create a direct access object rule.
    pub fn direct() -> ObjectRuleBuilder {
        ObjectRuleBuilder {
            access: ObjectAccess::Direct,
            fields: HashMap::new(),
        }
    }
}

/// Builder for object rules.
#[derive(Debug)]
pub struct ObjectRuleBuilder {
    access: ObjectAccess,
    fields: HashMap<String, FieldRule>,
}

impl ObjectRuleBuilder {
    /// Add a field rule.
    pub fn field(mut self, name: impl Into<String>, rule: FieldRule) -> Self {
        self.fields.insert(name.into(), rule);
        self
    }

    /// Build the object rule.
    pub fn build(self) -> ObjectRule {
        ObjectRule {
            access: self.access,
            fields: self.fields,
        }
    }
}

/// How to access a nested object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectAccess {
    /// Direct field access.
    Direct,
}
