use super::super::conditions::{decimal_integer, literal};
use crate::idents::{function_arg_expr, ident_key, unwrap_expr};
use sqlparser::ast::{CastKind, DataType, Expr, Function, FunctionArg, FunctionArguments, Value};

fn window_name(function: &Function) -> Option<String> {
    function.over.as_ref()?;
    let ident = match function.name.0.as_slice() {
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
        && matches!(args.get(1).and_then(function_arg_expr).and_then(|expr| integer_input(expr, false)),
            Some(Value::Number(offset, _)) if decimal_integer(offset) == Some(0))
    {
        return None;
    }
    function_arg_expr(args.first()?)
}

pub(in crate::statements::keyed_lookups) fn null_position_inputs(
    function: &Function,
) -> Option<&[FunctionArg]> {
    if !matches!(
        window_name(function).as_deref(),
        Some("nth_value" | "lag" | "lead")
    ) {
        return None;
    }
    let args = arguments(function)?;
    let [_, position, ..] = args else {
        return None;
    };
    matches!(
        integer_input(function_arg_expr(position)?, true),
        Some(Value::Null)
    )
    .then_some(args)
}

fn integer_input(expr: &Expr, allows_int4: bool) -> Option<&Value> {
    let mut position = unwrap_expr(expr);
    while let Expr::Cast {
        kind: CastKind::Cast | CastKind::DoubleColon,
        expr,
        data_type,
        format: None,
        ..
    } = position
    {
        if !(matches!(data_type, DataType::Int(None) | DataType::Integer(None))
            || allows_int4 && matches!(data_type, DataType::Int4(None)))
        {
            return None;
        }
        position = unwrap_expr(expr);
    }
    literal(position)
}
