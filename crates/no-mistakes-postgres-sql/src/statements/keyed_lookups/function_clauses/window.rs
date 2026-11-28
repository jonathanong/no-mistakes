use super::super::conditions::{decimal_integer, literal};
use crate::idents::{function_arg_expr, ident_key, unwrap_expr};
use sqlparser::ast::{DataType, Expr, Function, FunctionArg, FunctionArguments, Value};

fn window_name(function: &Function) -> Option<String> {
    function.over.as_ref()?;
    let ident = match function.name.0.as_slice() {
        [name] => name.as_ident(),
        [schema, name] if ident_key(schema.as_ident()?) == "pg_catalog" => name.as_ident(),
        _ => None,
    }?;
    Some(ident_key(ident))
}

fn arguments(function: &Function) -> Option<&[FunctionArg]> {
    let FunctionArguments::List(list) = &function.args else {
        return None;
    };
    Some(&list.args)
}

pub(in crate::statements::keyed_lookups) fn has_optional_window_default(
    function: &Function,
) -> bool {
    matches!(window_name(function).as_deref(), Some("lag" | "lead"))
}

pub(in crate::statements::keyed_lookups) fn row_window_value(function: &Function) -> Option<&Expr> {
    let name = window_name(function)?;
    if !matches!(
        name.as_str(),
        "lag" | "lead" | "first_value" | "last_value" | "nth_value"
    ) {
        return None;
    }
    let args = arguments(function)?;
    if matches!(name.as_str(), "lag" | "lead")
        && matches!(args.get(1).and_then(function_arg_expr).and_then(literal),
            Some(Value::Number(offset, _)) if decimal_integer(offset) == Some(0))
    {
        return None;
    }
    function_arg_expr(args.first()?)
}

pub(in crate::statements::keyed_lookups) fn nth_value_skipped_input(
    function: &Function,
) -> Option<&Expr> {
    if function.name.0.len() != 2 || window_name(function).as_deref()? != "nth_value" {
        return None;
    }
    let [value, position] = arguments(function)? else {
        return None;
    };
    let position = unwrap_expr(function_arg_expr(position)?);
    let position = match position {
        Expr::Cast {
            expr,
            data_type: DataType::Int(None) | DataType::Int4(None) | DataType::Integer(None),
            ..
        } => expr.as_ref(),
        other => other,
    };
    if matches!(literal(position), Some(Value::Null)) {
        function_arg_expr(value)
    } else {
        None
    }
}
