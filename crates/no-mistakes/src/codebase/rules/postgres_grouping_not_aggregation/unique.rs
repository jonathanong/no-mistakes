use super::{expressions::*, walk};
use crate::codebase::postgres::canonical_bound_expr;
use sqlparser::ast::*;
use std::collections::BTreeSet;
use std::ops::ControlFlow;
mod counts;
mod rows;
pub(super) fn redundant(select: &Select, catalog: &super::scope::CatalogScope<'_>) -> bool {
    let Some(relations) = walk::table_columns(select) else {
        return false;
    };
    if relations
        .iter()
        .any(|relation| catalog.relation(&relation.name).is_none())
    {
        return false;
    }
    let Some(projected) = projection(select) else {
        return false;
    };
    let keys = match &select.distinct {
        Some(Distinct::On(keys)) => Some(keys.iter().collect::<Vec<_>>()),
        Some(Distinct::Distinct) if !projected.iter().any(|expr| aggregate(expr)) => {
            Some(projected.clone())
        }
        _ if !groups(select).is_empty()
            && select.having.is_none()
            && projected.iter().all(|expr| {
                column(expr).is_some()
                    || catalog.pinned(expr)
                    || groups(select).iter().any(|key| bare(key) == bare(expr))
            }) =>
        {
            Some(groups(select).iter().collect())
        }
        _ => None,
    };
    if let Some(keys) = keys {
        return rows::proven(&relations, &keys, select, catalog, None);
    }
    counts::arguments(select).iter().any(|counted| {
        let mut keys = vec![counted];
        keys.extend(groups(select).iter());
        rows::proven(&relations, &keys, select, catalog, Some(counted))
    })
}
pub(super) fn key_covers(
    relation: &walk::Relation,
    keys: &[&Expr],
    select: &Select,
    catalog: &super::scope::CatalogScope<'_>,
    known: &BTreeSet<String>,
    (unqualified, nulls_ignored): (bool, Option<&Expr>),
) -> bool {
    let ignored_null_key = nulls_ignored
        .filter(|expr| !catalog.parameter(expr))
        .and_then(|expr| owned(expr, relation, unqualified));
    let predicates = walk::predicates(select);
    let mut covered = keys
        .iter()
        .filter(|expr| !catalog.parameter(expr))
        .filter_map(|expr| owned(expr, relation, unqualified))
        .collect::<Vec<_>>();
    let mut nonnull = Vec::new();
    for predicate in &predicates {
        match bare(predicate) {
            Expr::BinaryOp {
                left,
                op: BinaryOperator::Eq,
                right,
            } => {
                for (own, value) in [(left, right), (right, left)] {
                    if let Some(column) = (!catalog.parameter(own))
                        .then(|| owned(own, relation, unqualified))
                        .flatten()
                    {
                        nonnull.push(column.clone());
                        if catalog.pinned(value)
                            || super::expressions::column(value).is_some_and(|(alias, _)| {
                                alias.as_ref().is_some_and(|alias| known.contains(alias))
                            })
                        {
                            covered.push(column);
                        }
                    }
                }
            }
            Expr::IsNotNull(expr) => {
                if let Some(column) = (!catalog.parameter(expr))
                    .then(|| owned(expr, relation, unqualified))
                    .flatten()
                {
                    nonnull.push(column);
                }
            }
            _ => {}
        }
    }
    let qualifies = |key: &Expr, nulls_equal: bool| {
        covered.contains(key)
            && (nulls_equal
                || ignored_null_key.as_ref() == Some(key)
                || nonnull.contains(key)
                || match key {
                    Expr::Identifier(id) => catalog.column_is_not_null(&relation.name, &ident(id)),
                    _ => false,
                })
    };
    if catalog.unique_keys(&relation.name).iter().any(|keys| {
        !keys.is_empty()
            && keys
                .iter()
                .all(|name| qualifies(&Expr::Identifier(Ident::new(name)), false))
    }) {
        return true;
    }
    catalog
        .bounded_unique_indexes(&relation.name)
        .iter()
        .any(|index| {
            !index.keys.is_empty()
                && index.keys.iter().all(|key| {
                    qualifies(
                        &normalize(key, relation, true, None).unwrap_or_else(|| key.clone()),
                        index.nulls_not_distinct,
                    )
                }) && index.predicate.iter().all(|required| {
                normalize(required, relation, true, None).is_some_and(|required| {
                    predicates.iter().any(|predicate| {
                        normalize(predicate, relation, unqualified, Some(catalog.parameters)).as_ref() == Some(&required)
                    })
                }) || matches!(required, Expr::IsNotNull(expr) if normalize(expr, relation, true, None).is_some_and(|expr| nonnull.contains(&expr)))
            })
        })
}
fn owned(expr: &Expr, relation: &walk::Relation, unqualified: bool) -> Option<Expr> {
    let (alias, name) = column(expr)?;
    (alias.as_deref() == Some(&relation.alias) || alias.is_none() && unqualified)
        .then(|| Expr::Identifier(Ident::new(name)))
}
fn normalize(
    expr: &Expr,
    relation: &walk::Relation,
    unqualified: bool,
    parameters: Option<&[(u32, u32)]>,
) -> Option<Expr> {
    let mut expr = canonical_bound_expr(expr);
    let result = visit_expressions_mut(&mut expr, |expr| {
        if parameters.is_some_and(|parameters| super::scope::parameter(expr, parameters)) {
            return ControlFlow::Break(());
        }
        if let Some((_, name)) = column(expr) {
            if owned(expr, relation, unqualified).is_none() {
                return ControlFlow::Break(());
            }
            *expr = Expr::Identifier(Ident::new(name));
        }
        ControlFlow::Continue(())
    });
    result.is_continue().then_some(expr)
}
