use crate::codebase::postgres::idents::unwrap_expr;
use sqlparser::ast::{BinaryOperator, Expr, Value};

/// A false conjunct prevents every source row from matching, regardless of key pins.
pub(super) fn rejects_all(selection: Option<&Expr>) -> bool {
    selection.is_some_and(always_false)
}

fn always_false(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Value(value) => matches!(value.value, Value::Boolean(false)),
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => always_false(left) || always_false(right),
        _ => false,
    }
}

/// HAVING keeps only true groups; both false and SQL NULL reject a group.
pub(super) fn rejects_groups(having: Option<&Expr>) -> bool {
    having.is_some_and(group_rejected)
}

fn group_rejected(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Value(value) => matches!(value.value, Value::Boolean(false) | Value::Null),
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => group_rejected(left) || group_rejected(right),
        _ => false,
    }
}
