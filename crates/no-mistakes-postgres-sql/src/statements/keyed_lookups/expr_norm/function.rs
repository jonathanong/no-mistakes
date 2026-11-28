use super::super::builtins::SAFE_PURE_FUNCTIONS;
use super::argument_list;
use super::operators::comparable;
use super::types::has_builtin_type;
use crate::idents::{function_arg_expr, ident_key};
use sqlparser::ast::{Expr, Function, FunctionArg, FunctionArgExpr, FunctionArguments};

pub(super) fn normalize(
    function: &Function,
    mut norm_arg: impl FnMut(&Expr) -> Option<String>,
) -> Option<String> {
    let base = pure_name(function)?;
    let args = checked_arguments(function)?
        .iter()
        .map(|arg| norm_arg(proven_argument(arg)?))
        .collect::<Option<Vec<_>>>()?;
    Some(format!("fn({base},{})", argument_list(&args)))
}

pub(super) fn bare_arguments(function: &Function) -> Option<&[FunctionArg]> {
    let FunctionArguments::List(list) = &function.args else {
        return None;
    };
    (matches!(function.parameters, FunctionArguments::None)
        && list.clauses.is_empty()
        && list.duplicate_treatment.is_none())
    .then_some(list.args.as_slice())
}

fn proven_argument(arg: &FunctionArg) -> Option<&Expr> {
    match arg {
        FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) if has_builtin_type(expr) => Some(expr),
        _ => None,
    }
}

pub(super) fn arguments_are_proven(function: &Function) -> bool {
    checked_arguments(function)
        .is_some_and(|args| args.iter().all(|arg| proven_argument(arg).is_some()))
}

fn checked_arguments(function: &Function) -> Option<&[FunctionArg]> {
    let args = bare_arguments(function)?;
    if function
        .name
        .0
        .last()
        .and_then(|name| name.as_ident())
        .is_some_and(|name| name.value.eq_ignore_ascii_case("nullif"))
    {
        let [left, right] = args else {
            return None;
        };
        if !comparable(function_arg_expr(left)?, function_arg_expr(right)?) {
            return None;
        }
    }
    Some(args)
}

pub(in crate::statements::keyed_lookups) fn is_conditional_keyword(name: &str) -> bool {
    matches!(name, "coalesce" | "nullif" | "greatest" | "least")
}

pub(in crate::statements::keyed_lookups) fn pure_name(function: &Function) -> Option<String> {
    if function.over.is_some() || function.filter.is_some() || !function.within_group.is_empty() {
        return None;
    }
    let name = match function.name.0.as_slice() {
        [name] => {
            let ident = name.as_ident()?;
            let name = ident_key(ident);
            return (ident.quote_style.is_none() && is_conditional_keyword(&name)).then_some(name);
        }
        [schema, name] if ident_key(schema.as_ident()?) == "pg_catalog" => name,
        _ => return None,
    };
    let name = ident_key(name.as_ident()?);
    SAFE_PURE_FUNCTIONS.contains(&name.as_str()).then_some(name)
}
