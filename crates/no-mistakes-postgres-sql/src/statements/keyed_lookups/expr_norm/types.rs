use super::function::pure_name;
use crate::idents::unwrap_expr;
use sqlparser::ast::{ArrayElemTypeDef, BinaryOperator, DataType, Expr, Value};

pub(super) fn cast_type(data_type: &DataType) -> Option<String> {
    match data_type {
        DataType::Custom(..) | DataType::Array(ArrayElemTypeDef::None) => None,
        DataType::Array(
            ArrayElemTypeDef::AngleBracket(inner)
            | ArrayElemTypeDef::SquareBracket(inner, _)
            | ArrayElemTypeDef::Parenthesis(inner)
            | ArrayElemTypeDef::Qualified(inner, _),
        ) => Some(format!("{}[]", cast_type(inner)?)),
        other => Some(other.to_string().to_ascii_lowercase()),
    }
}

pub(super) fn cast_source_is_proven(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Value(value) => !matches!(value.value, Value::Placeholder(_)),
        Expr::Cast { .. } | Expr::TypedString(_) => true,
        Expr::Function(function) => pure_name(function).as_deref() == Some("md5"),
        _ => false,
    }
}

pub(super) fn is_custom_operator(op: &BinaryOperator) -> bool {
    matches!(
        op,
        BinaryOperator::Custom(_) | BinaryOperator::PGCustomBinaryOperator(_)
    )
}

pub(in crate::statements::keyed_lookups) fn projection_expr_is_unproven(expr: &Expr) -> bool {
    match expr {
        Expr::Function(function) => pure_name(function).is_none(),
        Expr::BinaryOp { op, .. } => is_custom_operator(op),
        Expr::Cast {
            expr, data_type, ..
        } => !cast_source_is_proven(expr) || cast_type(data_type).is_none(),
        Expr::TypedString(typed) => cast_type(&typed.data_type).is_none(),
        _ => false,
    }
}
