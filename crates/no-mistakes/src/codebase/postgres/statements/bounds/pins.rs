mod correlated;
mod resolver;

use super::{query, Scope};
use crate::codebase::postgres::idents::unwrap_expr;
use crate::codebase::postgres::statements::{SqlBoundPin, SqlPinSource};
use sqlparser::ast::{BinaryOperator, Expr, Query};
use std::collections::BTreeSet;

pub(super) use resolver::Resolver;

/// Collect the pins that the conjuncts of `expr` impose on the `restricted` items.
pub(super) fn extract(
    expr: &Expr,
    resolver: &Resolver,
    restricted: &[usize],
    scope: &Scope,
    out: &mut Vec<(usize, SqlBoundPin)>,
) {
    let mut pin = |column: &Expr, source: Option<SqlPinSource>, null_safe: bool| {
        if let (Some((item, column)), Some(source)) = (resolver.column(column), source) {
            if restricted.contains(&item) && resolver.is_table(item) {
                out.push((
                    item,
                    SqlBoundPin {
                        column,
                        source,
                        null_safe,
                    },
                ));
            }
        }
    };
    match unwrap_expr(expr) {
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => {
            extract(left, resolver, restricted, scope, out);
            extract(right, resolver, restricted, scope, out);
        }
        Expr::BinaryOp {
            left,
            op: BinaryOperator::Eq,
            right,
        } => equate(left, right, false, resolver, &mut pin),
        Expr::IsNotDistinctFrom(left, right) => equate(left, right, true, resolver, &mut pin),
        Expr::InList {
            expr,
            list,
            negated: false,
        } => {
            if let Some((item, _)) = resolver.column(expr) {
                let sources: Option<Vec<SqlPinSource>> = list
                    .iter()
                    .map(|value| resolver.source(value, item))
                    .collect();
                pin(expr, sources.map(merge), false);
            }
        }
        Expr::InSubquery {
            expr,
            subquery,
            negated: false,
        } => pin(expr, subquery_source(subquery, resolver, scope), false),
        Expr::AnyOp {
            left,
            compare_op: BinaryOperator::Eq,
            right,
            ..
        } => {
            if let Some((item, _)) = resolver.column(left) {
                let source = match unwrap_expr(right) {
                    Expr::Subquery(subquery) => subquery_source(subquery, resolver, scope),
                    other => resolver.source(other, item),
                };
                pin(left, source, false);
            }
        }
        _ => {}
    }
}

/// A subquery sizes the values it yields, unless it reads the row being checked: then every
/// row can find itself among them, whatever the subquery's own bound.
fn subquery_source(subquery: &Query, resolver: &Resolver, scope: &Scope) -> Option<SqlPinSource> {
    (!resolver.is_correlated(subquery))
        .then(|| SqlPinSource::Query(query::bound_query(subquery, scope)))
}

/// Pin either side of `left = right` that is a column of one item to the other side.
fn equate(
    left: &Expr,
    right: &Expr,
    null_safe: bool,
    resolver: &Resolver,
    pin: &mut impl FnMut(&Expr, Option<SqlPinSource>, bool),
) {
    if let Some((item, _)) = resolver.column(left) {
        pin(left, resolver.source(right, item), null_safe);
    }
    if let Some((item, _)) = resolver.column(right) {
        pin(right, resolver.source(left, item), null_safe);
    }
}

/// An `IN` list is sized by the caller, plus whatever other items its elements name.
fn merge(sources: Vec<SqlPinSource>) -> SqlPinSource {
    let items: BTreeSet<usize> = sources
        .into_iter()
        .flat_map(|source| match source {
            SqlPinSource::Items(items) => items,
            _ => Vec::new(),
        })
        .collect();
    if items.is_empty() {
        SqlPinSource::Value
    } else {
        SqlPinSource::Items(items.into_iter().collect())
    }
}
