//! Filter expressions for boolean queries.

use serde::{Deserialize, Serialize};

/// A filter expression AST for boolean queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterExpr {
  /// Field comparison: field op value
  Compare {
    field: String,
    op: CompareOp,
    value: FilterValue,
  },
  /// Boolean AND
  And(Vec<FilterExpr>),
  /// Boolean OR
  Or(Vec<FilterExpr>),
  /// Boolean NOT
  Not(Box<FilterExpr>),
}

/// Comparison operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompareOp {
  Eq,
  Ne,
  Lt,
  Le,
  Gt,
  Ge,
  Contains,
}

/// Values in filter expressions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterValue {
  String(String),
  Number(f64),
  Bool(bool),
}
