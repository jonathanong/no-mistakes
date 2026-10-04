use crate::codebase::postgres::idents::unwrap_expr;
use crate::codebase::postgres::statements::value::is_placeholder_ident;
use sqlparser::ast::{BinaryOperator, Expr, UnaryOperator, Value};

/// A boolean expression made only from scalar literals cannot expand an array's row count.
/// Calls and column references stay opaque: even a familiar function name may be overridden.
pub(super) fn fixed_scalar_boolean(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Value(value) if matches!(value.value, Value::Boolean(_) | Value::Null) => true,
        Expr::IsNull(inner)
        | Expr::IsNotNull(inner)
        | Expr::IsTrue(inner)
        | Expr::IsNotTrue(inner)
        | Expr::IsFalse(inner)
        | Expr::IsNotFalse(inner)
        | Expr::IsUnknown(inner)
        | Expr::IsNotUnknown(inner) => fixed_scalar_value(inner),
        Expr::UnaryOp {
            op: UnaryOperator::Not,
            expr,
        } => fixed_scalar_boolean(expr),
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And | BinaryOperator::Or | BinaryOperator::Xor,
            right,
        } => fixed_scalar_boolean(left) && fixed_scalar_boolean(right),
        Expr::BinaryOp {
            left,
            op:
                BinaryOperator::Eq
                | BinaryOperator::NotEq
                | BinaryOperator::Gt
                | BinaryOperator::GtEq
                | BinaryOperator::Lt
                | BinaryOperator::LtEq,
            right,
        }
        | Expr::IsDistinctFrom(left, right)
        | Expr::IsNotDistinctFrom(left, right) => {
            fixed_scalar_value(left) && fixed_scalar_value(right)
        }
        _ => false,
    }
}

fn fixed_scalar_value(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Value(_) | Expr::Interval(_) | Expr::TypedString(_) => true,
        Expr::Identifier(ident) if is_placeholder_ident(&ident.value) => true,
        Expr::UnaryOp {
            op: UnaryOperator::Plus | UnaryOperator::Minus,
            expr,
        } => fixed_scalar_value(expr),
        Expr::UnaryOp {
            op: UnaryOperator::Not,
            ..
        }
        | Expr::BinaryOp {
            op:
                BinaryOperator::And
                | BinaryOperator::Or
                | BinaryOperator::Xor
                | BinaryOperator::Eq
                | BinaryOperator::NotEq
                | BinaryOperator::Gt
                | BinaryOperator::GtEq
                | BinaryOperator::Lt
                | BinaryOperator::LtEq,
            ..
        }
        | Expr::IsNull(_)
        | Expr::IsNotNull(_)
        | Expr::IsTrue(_)
        | Expr::IsNotTrue(_)
        | Expr::IsFalse(_)
        | Expr::IsNotFalse(_)
        | Expr::IsUnknown(_)
        | Expr::IsNotUnknown(_)
        | Expr::IsDistinctFrom(_, _)
        | Expr::IsNotDistinctFrom(_, _) => fixed_scalar_boolean(expr),
        Expr::BinaryOp { left, right, .. } => fixed_scalar_value(left) && fixed_scalar_value(right),
        Expr::Cast { expr, .. } => fixed_scalar_value(expr),
        _ => false,
    }
}
