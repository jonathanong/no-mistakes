use super::Resolver;
use crate::codebase::postgres::idents::{ident_key, unwrap_expr};
use sqlparser::ast::{
    BinaryOperator, Expr, Function, FunctionArg, FunctionArgExpr, FunctionArguments, ObjectNamePart,
};

pub(in super::super) fn collect(
    expr: &Expr,
    resolver: &Resolver,
    restricted: &[usize],
    conjuncts: &mut Vec<(usize, Expr)>,
    nonnull: &mut Vec<(usize, String)>,
) {
    let expr = unwrap_expr(expr);
    if let Expr::BinaryOp {
        left,
        op: BinaryOperator::And,
        right,
    } = expr
    {
        collect(left, resolver, restricted, conjuncts, nonnull);
        collect(right, resolver, restricted, conjuncts, nonnull);
        return;
    }
    for &item in restricted {
        if resolver.is_table(item) {
            if let Some(canonical) = resolver.canonical_for_item(expr, item) {
                conjuncts.push((item, canonical));
            }
        }
    }
    if let Expr::BinaryOp {
        left,
        op: BinaryOperator::Eq,
        right,
    } = expr
    {
        for side in [left.as_ref(), right.as_ref()] {
            let column = resolver
                .column(side)
                .or_else(|| strict_column(side, resolver));
            if let Some((item, column)) = column {
                if restricted.contains(&item) && resolver.is_table(item) {
                    nonnull.push((item, column));
                }
            }
        }
    }
}

/// `lower`, `upper`, and one-argument `btrim` yield null when the argument is null.
/// See docs/rules/postgres-bounded-statements-proof.md.
fn strict_column(expr: &Expr, resolver: &Resolver) -> Option<(usize, String)> {
    let Expr::Function(function) = unwrap_expr(expr) else {
        return None;
    };
    let name = strict_name(function)?;
    if !matches!(name.as_str(), "btrim" | "lower" | "upper") {
        return None;
    }
    resolver.column(sole_argument(function)?)
}

fn strict_name(function: &Function) -> Option<String> {
    if function.over.is_some()
        || function.filter.is_some()
        || function.null_treatment.is_some()
        || !function.within_group.is_empty()
        || !matches!(function.parameters, FunctionArguments::None)
    {
        return None;
    }
    let parts = function
        .name
        .0
        .iter()
        .map(ObjectNamePart::as_ident)
        .collect::<Option<Vec<_>>>()?;
    match parts.as_slice() {
        [name] if name.quote_style.is_none() => Some(ident_key(name)),
        [schema, name]
            if schema.quote_style.is_none()
                && name.quote_style.is_none()
                && ident_key(schema) == "pg_catalog" =>
        {
            Some(ident_key(name))
        }
        _ => None,
    }
}

fn sole_argument(function: &Function) -> Option<&Expr> {
    let FunctionArguments::List(arguments) = &function.args else {
        return None;
    };
    if arguments.args.len() != 1
        || arguments.duplicate_treatment.is_some()
        || !arguments.clauses.is_empty()
    {
        return None;
    }
    let Some(FunctionArg::Unnamed(FunctionArgExpr::Expr(argument))) = arguments.args.first() else {
        return None;
    };
    Some(argument)
}

#[cfg(test)]
#[path = "evidence_tests.rs"]
mod evidence_tests;
