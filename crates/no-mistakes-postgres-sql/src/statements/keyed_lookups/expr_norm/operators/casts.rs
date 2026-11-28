use super::{operand_type, type_name, BuiltinType};
use crate::idents::unwrap_expr;
use sqlparser::ast::{ArrayElemTypeDef, DataType, Expr, Value};

pub(in crate::statements::keyed_lookups::expr_norm) fn cast_is_proven(
    expr: &Expr,
    target: &DataType,
) -> bool {
    if let Expr::Value(value) = unwrap_expr(expr) {
        if matches!(
            value.value,
            Value::Null | Value::SingleQuotedString(_) | Value::EscapedStringLiteral(_)
        ) {
            return known_cast_target(target);
        }
    }
    let Some(source) = operand_type(expr) else {
        return false;
    };
    if matches!(
        target,
        DataType::Text | DataType::Varchar(_) | DataType::CharacterVarying(_)
    ) || source == BuiltinType::Text && matches!(target, DataType::Uuid)
    {
        return true;
    }
    type_name(target).is_some_and(|target| {
        source == target
            || source.numeric() && target.numeric()
            || matches!(
                (source, target),
                (BuiltinType::Bool, BuiltinType::Integer)
                    | (BuiltinType::Integer, BuiltinType::Bool)
            )
    })
}

pub(in crate::statements::keyed_lookups::expr_norm) fn known_cast_target(
    target: &DataType,
) -> bool {
    match target {
        DataType::Array(
            ArrayElemTypeDef::SquareBracket(inner, _)
            | ArrayElemTypeDef::AngleBracket(inner)
            | ArrayElemTypeDef::Parenthesis(inner)
            | ArrayElemTypeDef::Qualified(inner, _),
        ) => known_cast_target(inner),
        DataType::Varchar(_)
        | DataType::CharacterVarying(_)
        | DataType::Char(_)
        | DataType::Character(_)
        | DataType::Uuid
        | DataType::Bytea
        | DataType::JSON
        | DataType::JSONB => true,
        other => type_name(other).is_some(),
    }
}
