use super::function::{arguments_are_proven, pure_name};
use super::operators::{
    array_elements_are_proven, binary_is_proven, case_is_proven, cast_is_proven, comparable,
    is_proven_boolean, is_proven_text, known_cast_target, list_operands_are_proven,
    pattern_is_proven, substring_is_proven, unary_is_proven,
};
use crate::idents::unwrap_expr;
use sqlparser::ast::{ArrayElemTypeDef, CastKind, DataType, Expr, InTarget, Value};

pub(super) fn cast_type(data_type: &DataType) -> Option<String> {
    match data_type {
        DataType::Custom(..) | DataType::Array(ArrayElemTypeDef::None) => None,
        DataType::Array(
            ArrayElemTypeDef::AngleBracket(inner)
            | ArrayElemTypeDef::SquareBracket(inner, _)
            | ArrayElemTypeDef::Parenthesis(inner)
            | ArrayElemTypeDef::Qualified(inner, _),
        ) => Some(format!("{}[]", cast_type(inner)?)),
        other => Some(
            integer_type_alias(other)
                .unwrap_or(other)
                .to_string()
                .to_ascii_lowercase(),
        ),
    }
}

pub(in crate::statements::keyed_lookups) fn integer_type_alias(
    data_type: &DataType,
) -> Option<&'static DataType> {
    matches!(data_type, DataType::Int(None)).then_some(&DataType::Integer(None))
}

pub(super) fn has_builtin_type(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Value(value) => !matches!(value.value, Value::Placeholder(_)),
        Expr::Cast {
            kind: CastKind::Cast | CastKind::DoubleColon,
            expr,
            data_type,
            format: None,
            ..
        } => cast_is_proven(expr, data_type),
        Expr::TypedString(typed) => known_cast_target(&typed.data_type),
        Expr::IsNull(_) | Expr::IsNotNull(_) => true,
        Expr::Function(function) => pure_name(function).is_some() && arguments_are_proven(function),
        Expr::BinaryOp { left, op, right } => binary_is_proven(op, left, right),
        Expr::UnaryOp { op, expr } => unary_is_proven(*op, expr),
        _ => false,
    }
}

pub(in crate::statements::keyed_lookups) fn projection_expr_is_unproven(expr: &Expr) -> bool {
    match expr {
        Expr::Function(function) => {
            pure_name(function).is_none() || !arguments_are_proven(function)
        }
        Expr::BinaryOp { left, op, right } => !binary_is_proven(op, left, right),
        Expr::UnaryOp { op, expr } => !unary_is_proven(*op, expr),
        Expr::IsTrue(expr)
        | Expr::IsNotTrue(expr)
        | Expr::IsFalse(expr)
        | Expr::IsNotFalse(expr)
        | Expr::IsUnknown(expr)
        | Expr::IsNotUnknown(expr) => !is_proven_boolean(expr),
        Expr::IsDistinctFrom(left, right) | Expr::IsNotDistinctFrom(left, right) => {
            !comparable(left, right)
        }
        Expr::In {
            expr,
            target: InTarget::List(items),
            ..
        } => !list_operands_are_proven(expr, items),
        Expr::Between {
            expr, low, high, ..
        } => !comparable(expr, low) || !comparable(expr, high),
        Expr::Like {
            expr,
            pattern,
            escape_char,
            any,
            ..
        }
        | Expr::ILike {
            expr,
            pattern,
            escape_char,
            any,
            ..
        } => *any || !pattern_is_proven(expr, pattern, escape_char.as_deref()),
        Expr::SimilarTo {
            expr,
            pattern,
            escape_char,
            ..
        } => !pattern_is_proven(expr, pattern, escape_char.as_deref()),
        Expr::Trim {
            expr,
            trim_what,
            trim_characters,
            ..
        } => {
            !is_proven_text(expr)
                || trim_what
                    .as_deref()
                    .is_some_and(|expr| !is_proven_text(expr))
                || trim_characters
                    .iter()
                    .flatten()
                    .any(|expr| !is_proven_text(expr))
        }
        Expr::Substring {
            expr,
            substring_from,
            substring_for,
            shorthand,
            special,
        } => {
            *shorthand
                || *special
                || !substring_is_proven(
                    expr,
                    substring_from.as_deref(),
                    substring_for.as_deref(),
                    true,
                )
        }
        Expr::Case {
            operand,
            conditions,
            else_result,
            ..
        } => !case_is_proven(operand.as_deref(), conditions, else_result.as_deref()),
        Expr::Array(array) => !array_elements_are_proven(&array.elem),
        Expr::Cast { .. } | Expr::TypedString(_) => !has_builtin_type(expr),
        Expr::Identifier(_)
        | Expr::CompoundIdentifier(_)
        | Expr::Value(_)
        | Expr::Nested(_)
        | Expr::IsNull(_)
        | Expr::IsNotNull(_) => false,
        _ => true,
    }
}
