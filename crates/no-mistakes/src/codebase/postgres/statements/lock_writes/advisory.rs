use super::helpers::constant;
use crate::codebase::postgres::idents::{object_name_identity, unwrap_expr};
use sqlparser::ast::{Expr, FunctionArg, FunctionArgExpr, FunctionArguments, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AdvisoryKey {
    Constant(String),
    Parameter(String),
    Hash(String, Vec<AdvisoryKey>),
}

pub(super) fn keys(expr: &Expr, hashed: bool) -> Option<Vec<AdvisoryKey>> {
    let Expr::Function(function) = expr else {
        return None;
    };
    let name = object_name_identity(&function.name);
    let name = name.strip_prefix("pg_catalog.").unwrap_or(&name);
    if !matches!(
        name,
        "pg_advisory_lock"
            | "pg_advisory_xact_lock"
            | "pg_try_advisory_lock"
            | "pg_try_advisory_xact_lock"
            | "pg_advisory_lock_shared"
            | "pg_advisory_xact_lock_shared"
            | "pg_try_advisory_lock_shared"
            | "pg_try_advisory_xact_lock_shared"
    ) {
        return None;
    }
    let args = arguments(&function.args, hashed)?;
    matches!(args.len(), 1 | 2).then_some(args)
}

fn arguments(args: &FunctionArguments, hashed: bool) -> Option<Vec<AdvisoryKey>> {
    let FunctionArguments::List(args) = args else {
        return None;
    };
    args.args
        .iter()
        .map(|arg| match arg {
            FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) => key(expr, hashed),
            _ => None,
        })
        .collect()
}

fn key(expr: &Expr, hashed: bool) -> Option<AdvisoryKey> {
    if constant(expr) {
        return Some(AdvisoryKey::Constant(expr.to_string()));
    }
    match unwrap_expr(expr) {
        Expr::Value(value) if matches!(value.value, Value::Placeholder(_)) => {
            Some(AdvisoryKey::Parameter(value.to_string()))
        }
        Expr::Function(function) if hashed => {
            let name = object_name_identity(&function.name);
            let name = name.strip_prefix("pg_catalog.").unwrap_or(&name);
            let arity = match name {
                "hashtext" => 1,
                "hashtextextended" => 2,
                _ => return None,
            };
            let args = arguments(&function.args, hashed)?;
            (args.len() == arity).then(|| AdvisoryKey::Hash(name.to_owned(), args))
        }
        Expr::Cast { expr, .. } => key(expr, hashed),
        _ => None,
    }
}
