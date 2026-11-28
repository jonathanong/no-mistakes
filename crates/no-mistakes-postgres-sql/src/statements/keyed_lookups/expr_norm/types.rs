use super::function::{arguments_are_proven, pure_name};
use crate::idents::unwrap_expr;
use sqlparser::ast::{
    ArrayElemTypeDef, BinaryOperator, DataType, Expr, InTarget, UnaryOperator, Value,
};

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

pub(super) fn has_builtin_type(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Value(value) => !matches!(value.value, Value::Placeholder(_)),
        Expr::Cast { .. } | Expr::TypedString(_) | Expr::IsNull(_) | Expr::IsNotNull(_) => true,
        Expr::Function(function) => pure_name(function).is_some() && arguments_are_proven(function),
        Expr::BinaryOp { left, op, right } => binary_is_proven(op, left, right),
        Expr::UnaryOp { op, expr } => unary_is_proven(*op, expr),
        _ => false,
    }
}

pub(super) fn binary_is_proven(op: &BinaryOperator, left: &Expr, right: &Expr) -> bool {
    !matches!(
        op,
        BinaryOperator::Custom(_) | BinaryOperator::PGCustomBinaryOperator(_)
    ) && has_builtin_type(left)
        && has_builtin_type(right)
}

pub(super) fn list_operands_are_proven(expr: &Expr, items: &[Expr]) -> bool {
    has_builtin_type(expr) && items.iter().all(has_builtin_type)
}

pub(super) fn unary_is_proven(op: UnaryOperator, expr: &Expr) -> bool {
    matches!(
        op,
        UnaryOperator::Plus | UnaryOperator::Minus | UnaryOperator::Not | UnaryOperator::BitwiseNot
    ) && has_builtin_type(expr)
}

pub(in crate::statements::keyed_lookups) fn projection_expr_is_unproven(expr: &Expr) -> bool {
    match expr {
        Expr::Function(function) => {
            pure_name(function).is_none() || !arguments_are_proven(function)
        }
        Expr::BinaryOp { left, op, right } => !binary_is_proven(op, left, right),
        Expr::UnaryOp { op, expr } => !unary_is_proven(*op, expr),
        Expr::IsDistinctFrom(left, right) | Expr::IsNotDistinctFrom(left, right) => {
            !has_builtin_type(left) || !has_builtin_type(right)
        }
        Expr::In {
            expr,
            target: InTarget::List(items),
            ..
        } => !list_operands_are_proven(expr, items),
        Expr::Cast {
            expr, data_type, ..
        } => !has_builtin_type(expr) || cast_type(data_type).is_none(),
        Expr::TypedString(typed) => cast_type(&typed.data_type).is_none(),
        _ => false,
    }
}
