use super::super::conditions::literal;
use super::super::expr_norm::function::{bare_arguments, positional_argument};
use crate::idents::{ident_key, unwrap_expr};
use sqlparser::ast::{CastKind, DataType, Expr, Function, FunctionArg, Value};

pub(in crate::statements::keyed_lookups) fn strict_null_arguments(
    function: &Function,
) -> Option<&[FunctionArg]> {
    let [schema, name] = function.name.0.as_slice() else {
        return None;
    };
    if ident_key(schema.as_ident()?) != "pg_catalog"
        || ident_key(name.as_ident()?) != "replace"
        || function.filter.is_some()
        || function.over.is_some()
        || !function.within_group.is_empty()
    {
        return None;
    }
    let arguments @ [first, second, third] = bare_arguments(function)? else {
        return None;
    };
    let inputs = [first, second, third].map(|arg| positional_argument(arg).and_then(text_is_null));
    (inputs.iter().all(Option::is_some) && inputs.contains(&Some(true))).then_some(arguments)
}

fn text_is_null(expr: &Expr) -> Option<bool> {
    match unwrap_expr(expr) {
        Expr::Cast {
            kind: CastKind::Cast | CastKind::DoubleColon,
            expr,
            data_type: DataType::Text,
            format: None,
            ..
        } => Some(matches!(literal(expr), Some(Value::Null))),
        Expr::TypedString(typed) if typed.data_type == DataType::Text => Some(false),
        Expr::Value(value) => match value.value {
            Value::Null => Some(true),
            Value::SingleQuotedString(_) | Value::EscapedStringLiteral(_) => Some(false),
            _ => None,
        },
        _ => None,
    }
}
