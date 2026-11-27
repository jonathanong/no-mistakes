use super::super::builtins::SAFE_PURE_FUNCTIONS;
use super::argument_list;
use crate::idents::ident_key;
use sqlparser::ast::{Expr, Function, FunctionArg, FunctionArgExpr, FunctionArguments};

pub(super) fn normalize(
    function: &Function,
    mut norm_arg: impl FnMut(&Expr) -> Option<String>,
) -> Option<String> {
    let base = pure_name(function)?;
    let FunctionArguments::List(list) = &function.args else {
        return None;
    };
    if !list.clauses.is_empty() || list.duplicate_treatment.is_some() {
        return None;
    }
    let args = list
        .args
        .iter()
        .map(|arg| match arg {
            FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) => norm_arg(expr),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    Some(format!("fn({base},{})", argument_list(&args)))
}

pub(in crate::statements::keyed_lookups) fn pure_name(function: &Function) -> Option<String> {
    if function.over.is_some() || function.filter.is_some() || !function.within_group.is_empty() {
        return None;
    }
    let name = match function.name.0.as_slice() {
        [name] => {
            let ident = name.as_ident()?;
            let name = ident_key(ident);
            return (ident.quote_style.is_none()
                && matches!(name.as_str(), "coalesce" | "nullif" | "greatest" | "least"))
            .then_some(name);
        }
        [schema, name] if ident_key(schema.as_ident()?) == "pg_catalog" => name,
        _ => return None,
    };
    let name = ident_key(name.as_ident()?);
    SAFE_PURE_FUNCTIONS.contains(&name.as_str()).then_some(name)
}
