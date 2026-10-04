use crate::codebase::postgres::idents::unwrap_expr;
use sqlparser::ast::{DataType, Expr, UnaryOperator, Value};

pub(in super::super) fn constructor(expr: &Expr) -> Option<&sqlparser::ast::Array> {
    match unwrap_expr(expr) {
        Expr::Array(array) => Some(array),
        Expr::Cast {
            expr,
            data_type: DataType::Array(_),
            ..
        } => constructor(expr),
        _ => None,
    }
}

// Numeric signs do not change scalar cardinality; arbitrary overloaded operators need proof.
pub(super) fn numeric_literal(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Value(value) => matches!(&value.value, Value::Number(..)),
        Expr::UnaryOp {
            op: UnaryOperator::Plus | UnaryOperator::Minus,
            expr,
        } => numeric_literal(expr),
        _ => false,
    }
}
