use crate::codebase::postgres::idents::unwrap_expr;
use sqlparser::ast::{BinaryOperator, Expr, Value};

/// `TRUE`, or a literal equated with itself (`1 = 1`): it never narrows anything.
pub(super) fn is_constant_true(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Value(value) => matches!(value.value, Value::Boolean(true)),
        Expr::BinaryOp {
            left,
            op: BinaryOperator::Eq,
            right,
        } => match (unwrap_expr(left), unwrap_expr(right)) {
            (Expr::Value(left), Expr::Value(right)) => {
                left.value == right.value
                    && !matches!(left.value, Value::Null | Value::Placeholder(_))
            }
            _ => false,
        },
        _ => false,
    }
}

pub(super) fn flatten<'a>(expr: &'a Expr, out: &mut Vec<&'a Expr>) {
    match unwrap_expr(expr) {
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => {
            flatten(left, out);
            flatten(right, out);
        }
        other => out.push(other),
    }
}
