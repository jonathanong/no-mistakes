use super::numeric_literal;
use sqlparser::ast::{DataType, Expr, UnaryOperator, Value};

/// Only built-in numeric wrappers prove zero; a user-defined cast may change its value.
pub(super) fn is_zero(expr: &Expr) -> bool {
    match expr {
        Expr::Nested(expr)
        | Expr::UnaryOp {
            op: UnaryOperator::Plus | UnaryOperator::Minus,
            expr,
        } => is_zero(expr),
        Expr::Cast {
            expr, data_type, ..
        } if numeric_type(data_type) => is_zero(expr),
        Expr::Value(value) => match &value.value {
            Value::Number(text, _) => numeric_literal(text).is_ok_and(|value| value == 0),
            _ => false,
        },
        _ => false,
    }
}

fn numeric_type(data_type: &DataType) -> bool {
    matches!(
        data_type,
        DataType::BigInt(_)
            | DataType::Int(_)
            | DataType::Int2(_)
            | DataType::Int4(_)
            | DataType::Int8(_)
            | DataType::Integer(_)
            | DataType::SmallInt(_)
            | DataType::Numeric(_)
            | DataType::Decimal(_)
            | DataType::Dec(_)
            | DataType::Real
            | DataType::DoublePrecision
            | DataType::Float(_)
            | DataType::Float4
            | DataType::Float8
    )
}
