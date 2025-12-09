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

impl FilterExpr {
  /// Evaluates the filter expression against a given item.
  ///
  /// The item must implement `serde::Serialize` so that its fields can be
  /// accessed dynamically.
  pub fn evaluate<T: serde::Serialize>(&self, item: &T) -> bool {
    let json_value = match serde_json::to_value(item) {
      Ok(v) => v,
      Err(_) => return false,
    };

    self.evaluate_value(&json_value)
  }

  fn evaluate_value(&self, item: &serde_json::Value) -> bool {
    match self {
      FilterExpr::Compare { field, op, value } => {
        let field_value = get_field_value(item, field);
        compare_values(field_value, op, value)
      }
      FilterExpr::And(exprs) => exprs.iter().all(|e| e.evaluate_value(item)),
      FilterExpr::Or(exprs) => exprs.iter().any(|e| e.evaluate_value(item)),
      FilterExpr::Not(expr) => !expr.evaluate_value(item),
    }
  }
}

/// Helper function to get a value from a nested JSON object using dot notation.
fn get_field_value<'a>(item: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
  let mut current = item;
  for part in path.split('.') {
    current = current.get(part)?;
  }
  Some(current)
}

/// Helper function to compare a JSON value from the item against a filter value.
fn compare_values(
  field_value: Option<&serde_json::Value>,
  op: &CompareOp,
  target_value: &FilterValue,
) -> bool {
  let field_value = match field_value {
    Some(v) => v,
    None => return false,
  };

  match (field_value, target_value) {
    (serde_json::Value::String(s), FilterValue::String(t)) if *op == CompareOp::Contains => {
      s.contains(t)
    }
    (serde_json::Value::String(s), FilterValue::String(t)) => compare_ord(s, op, t),
    (serde_json::Value::Number(n), FilterValue::Number(t)) => {
      if let Some(f) = n.as_f64() {
        compare_ord(&f, op, t)
      } else {
        false
      }
    }
    (serde_json::Value::Bool(b), FilterValue::Bool(t)) => match op {
      CompareOp::Eq => b == t,
      CompareOp::Ne => b != t,
      _ => false,
    },
    (serde_json::Value::Array(arr), target) => match op {
      CompareOp::Contains => arr.iter().any(|elem| match (elem, target) {
        (serde_json::Value::String(s), FilterValue::String(t)) => s == t,
        (serde_json::Value::Number(n), FilterValue::Number(t)) => n.as_f64() == Some(*t),
        (serde_json::Value::Bool(b), FilterValue::Bool(t)) => b == t,
        _ => false,
      }),
      _ => false,
    },
    _ => false,
  }
}

fn compare_ord<T: PartialOrd>(a: &T, op: &CompareOp, b: &T) -> bool {
  match op {
    CompareOp::Eq => a == b,
    CompareOp::Ne => a != b,
    CompareOp::Lt => a < b,
    CompareOp::Le => a <= b,
    CompareOp::Gt => a > b,
    CompareOp::Ge => a >= b,
    CompareOp::Contains => false,
  }
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
