//! Defines the structures for building filter expressions for queries.
//!
//! This module provides an Abstract Syntax Tree (AST) for constructing
//! boolean logic filters that can be applied to search queries. This allows
//! for more complex, structured filtering beyond simple keyword matching.

use serde::{Deserialize, Serialize};

/// An enum representing the nodes of a filter expression AST.
///
/// This AST allows for the creation of complex boolean queries involving
/// field comparisons, AND, OR, and NOT operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterExpr {
  /// A comparison between a field and a value.
  ///
  /// This is the leaf node of the expression tree, representing a condition
  /// like "price < 50" or "category == 'electronics'".
  Compare {
    /// The name of the field to compare, which can be nested (e.g., "author.name").
    field: String,
    /// The comparison operator to use.
    op: CompareOp,
    /// The value to compare against.
    value: FilterValue,
  },
  /// A logical AND operation.
  ///
  /// The expression is true only if all the sub-expressions in the vector are true.
  And(Vec<FilterExpr>),
  /// A logical OR operation.
  ///
  /// The expression is true if at least one of the sub-expressions in the vector is true.
  Or(Vec<FilterExpr>),
  /// A logical NOT operation.
  ///
  /// The expression inverts the result of the sub-expression.
  Not(Box<FilterExpr>),
}

/// The set of comparison operators available for filter expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompareOp {
  /// Equal to (`==`)
  Eq,
  /// Not equal to (`!=`)
  Ne,
  /// Less than (`<`)
  Lt,
  /// Less than or equal to (`<=`)
  Le,
  /// Greater than (`>`)
  Gt,
  /// Greater than or equal to (`>=`)
  Ge,
  /// Contains (for strings and arrays)
  Contains,
}

/// Represents the possible types of values used in filter expressions.
///
/// The `#[serde(untagged)]` attribute allows for flexible deserialization from
/// JSON, as it will try to match the value to one of the variants without
/// requiring a specific tag.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterValue {
  /// A string value.
  String(String),
  /// A floating-point number value.
  Number(f64),
  /// A boolean value.
  Bool(bool),
}
