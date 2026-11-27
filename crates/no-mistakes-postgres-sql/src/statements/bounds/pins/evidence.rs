use super::Resolver;
use crate::idents::{ident_key, unwrap_expr};
use sqlparser::ast::{
    BinaryOperator, Expr, Function, FunctionArg, FunctionArgExpr, FunctionArguments,
    ObjectNamePart, Value,
};

pub(in super::super) fn collect(
    expr: &Expr,
    resolver: &Resolver,
    restricted: &[usize],
    conjuncts: &mut Vec<(usize, Expr)>,
    nonnull: &mut Vec<(usize, String)>,
    strict_nonnull: &mut Vec<(usize, String)>,
) {
    let expr = unwrap_expr(expr);
    if let Expr::BinaryOp {
        left,
        op: BinaryOperator::And,
        right,
    } = expr
    {
        collect(
            left,
            resolver,
            restricted,
            conjuncts,
            nonnull,
            strict_nonnull,
        );
        collect(
            right,
            resolver,
            restricted,
            conjuncts,
            nonnull,
            strict_nonnull,
        );
        return;
    }
    for &item in restricted {
        if resolver.is_table(item) {
            for conjunct in item_predicates(resolver, expr, item) {
                conjuncts.push((item, conjunct));
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
            if let Some((item, column)) = resolver.column(side) {
                if restricted.contains(&item) && resolver.is_table(item) {
                    nonnull.push((item, column));
                }
            } else if let Some((item, column)) = strict_column(side, resolver) {
                if restricted.contains(&item) && resolver.is_table(item) {
                    strict_nonnull.push((item, column));
                }
            }
        }
    }
}

/// Column conjuncts of `expr` for `item`, plus a placeholder-to-literal binding.
pub(in super::super) fn item_predicates(
    resolver: &Resolver,
    expr: &Expr,
    item: usize,
) -> Vec<Expr> {
    let mut found = Vec::new();
    if let Some(canonical) = resolver.canonical_for_item(expr, item) {
        found.push(canonical);
    }
    if let Some(binding) = placeholder_binding(expr) {
        found.push(binding);
    }
    found
}

fn placeholder_binding(expr: &Expr) -> Option<Expr> {
    let Expr::BinaryOp {
        left,
        op: BinaryOperator::Eq,
        right,
    } = unwrap_expr(expr)
    else {
        return None;
    };
    let matched = [
        (left.as_ref(), right.as_ref()),
        (right.as_ref(), left.as_ref()),
    ]
    .into_iter()
    .any(|(placeholder, literal)| is_placeholder(placeholder) && is_literal(literal));
    matched.then(|| crate::canonical_bound_expr(unwrap_expr(expr)))
}

fn is_placeholder(expr: &Expr) -> bool {
    matches!(
        unwrap_expr(expr),
        Expr::Value(value) if matches!(value.value, Value::Placeholder(_))
    )
}

fn is_literal(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Value(value) => matches!(
            value.value,
            Value::SingleQuotedString(_) | Value::Boolean(_) | Value::Number(_, _)
        ),
        Expr::Cast { expr, .. } => is_literal(expr),
        _ => false,
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

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
#[path = "evidence_tests.rs"]
mod evidence_tests;
