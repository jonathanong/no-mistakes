mod eager;

use super::conditions::{literal, literal_condition};
use super::select_exprs::target_identity;
use crate::fx::FxHashSet;
use crate::idents::{function_arg_expr, ident_key, unwrap_expr, visit_shallow_exprs};
pub(super) use eager::visit_eager_functions;
use sqlparser::ast::{
    DataType, DuplicateTreatment, Expr, Function, FunctionArg, FunctionArgumentClause,
    FunctionArguments, Value,
};

pub(super) fn coalesce_arguments(function: &Function) -> Option<&[FunctionArg]> {
    let [name] = function.name.0.as_slice() else {
        return None;
    };
    let name = name.as_ident()?;
    if name.quote_style.is_some()
        || !name.value.eq_ignore_ascii_case("coalesce")
        || is_eager(function)
    {
        return None;
    }
    match &function.args {
        FunctionArguments::List(list) => Some(&list.args),
        _ => None,
    }
}

pub(super) fn visit_function_inputs(function: &Function, visit: &mut impl FnMut(&Expr)) {
    visit_arguments(&function.parameters, usize::MAX, visit);
    if filter_skips_inputs(function) {
        if function.over.is_some() {
            visit_arguments(&function.args, usize::MAX, &mut |argument| {
                visit_eager_functions(argument, visit);
            });
        }
        return;
    }
    if let Some(value) = nth_value_skipped_input(function) {
        visit_eager_functions(value, visit);
        return;
    }
    let has_default = has_optional_window_default(function);
    let argument_limit = if has_default { 2 } else { usize::MAX };
    if function.within_group.is_empty() {
        visit_arguments(&function.args, argument_limit, visit);
        if has_default {
            if let FunctionArguments::List(list) = &function.args {
                if let Some(default) = list.args.get(2).and_then(function_arg_expr) {
                    visit_eager_functions(default, visit);
                }
            }
        }
    }
    visit_shallow_exprs(&function.within_group, visit);
}

pub(super) fn visit_ordered_set_arguments(function: &Function, visit: &mut impl FnMut(&Expr)) {
    if !function.within_group.is_empty() {
        visit_arguments(&function.args, usize::MAX, visit);
    }
}

pub(super) fn split_coalesce_arguments(
    arguments: &[FunctionArg],
) -> (&[FunctionArg], &[FunctionArg]) {
    let evaluated = arguments.iter().position(|argument| {
        matches!(function_arg_expr(argument).map(unwrap_expr), Some(Expr::Value(value))
            if !matches!(value.value, Value::Null | Value::Placeholder(_)))
    });
    arguments.split_at(evaluated.map_or(arguments.len(), |index| index + 1))
}

pub(super) fn filter_skips_inputs(function: &Function) -> bool {
    function.filter.as_deref().and_then(literal_condition) == Some(false)
}

pub(super) fn has_optional_window_default(function: &Function) -> bool {
    function.over.is_some()
        && match function.name.0.as_slice() {
            [name] => name.as_ident(),
            [schema, name]
                if schema
                    .as_ident()
                    .is_some_and(|s| ident_key(s) == "pg_catalog") =>
            {
                name.as_ident()
            }
            _ => None,
        }
        .is_some_and(|name| matches!(ident_key(name).as_str(), "lag" | "lead"))
}

pub(super) fn nth_value_skipped_input(function: &Function) -> Option<&Expr> {
    let [schema, name] = function.name.0.as_slice() else {
        return None;
    };
    if function.over.is_none()
        || ident_key(schema.as_ident()?) != "pg_catalog"
        || ident_key(name.as_ident()?) != "nth_value"
    {
        return None;
    }
    let FunctionArguments::List(list) = &function.args else {
        return None;
    };
    let [value, position] = list.args.as_slice() else {
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
    matches!(literal(position), Some(Value::Null)).then(|| function_arg_expr(value))?
}

fn visit_arguments(
    arguments: &FunctionArguments,
    argument_limit: usize,
    visit: &mut impl FnMut(&Expr),
) {
    let FunctionArguments::List(list) = arguments else {
        return;
    };
    for arg in list
        .args
        .iter()
        .take(argument_limit)
        .filter_map(function_arg_expr)
    {
        visit(arg);
    }
    for clause in &list.clauses {
        if let FunctionArgumentClause::OrderBy(order_by) = clause {
            let mut targets: FxHashSet<_> = list
                .args
                .iter()
                .filter_map(function_arg_expr)
                .map(target_identity)
                .collect();
            for item in order_by {
                if targets.insert(target_identity(&item.expr)) {
                    visit(&item.expr);
                }
            }
        } else {
            visit_shallow_exprs(clause, visit);
        }
    }
}

pub(super) fn is_eager(function: &sqlparser::ast::Function) -> bool {
    if function.filter.is_some() || function.over.is_some() || !function.within_group.is_empty() {
        return true;
    }
    if let FunctionArguments::List(list) = &function.args {
        if list.duplicate_treatment == Some(DuplicateTreatment::Distinct)
            || list
                .clauses
                .iter()
                .any(|clause| matches!(clause, FunctionArgumentClause::OrderBy(_)))
        {
            return true;
        }
    }
    let [schema, name] = function.name.0.as_slice() else {
        return false;
    };
    schema
        .as_ident()
        .is_some_and(|schema| ident_key(schema) == "pg_catalog")
        && name.as_ident().is_some_and(|name| {
            matches!(
                ident_key(name).as_str(),
                "min"
                    | "max"
                    | "sum"
                    | "avg"
                    | "count"
                    | "array_agg"
                    | "string_agg"
                    | "bool_and"
                    | "bool_or"
                    | "json_agg"
                    | "jsonb_agg"
            )
        })
}
