//! Shared ANY equality value evidence for resolved and catalog-owned bare pins.
use super::array::{caller_array, constructor, finite_array, scalar_subquery};
use super::{subquery_source, Resolver, Scope, Sourced};
use crate::codebase::postgres::idents::unwrap_expr;
use crate::codebase::postgres::statements::SqlPinSource;
use sqlparser::ast::Expr;

pub(super) fn query(
    right: &Expr,
    resolver: &Resolver,
    scope: &Scope,
    positions: super::super::super::value::PlaceholderPositions<'_>,
) -> Option<Sourced> {
    match unwrap_expr(right) {
        Expr::Subquery(subquery) => Some(subquery_source(subquery, resolver, scope, positions)),
        other => {
            let subquery = scalar_subquery(other)?;
            let mut sourced = subquery_source(subquery, resolver, scope, positions);
            if let SqlPinSource::Query(query) = sourced.source {
                sourced.source = SqlPinSource::ReadQuery(query);
            }
            Some(sourced)
        }
    }
}

pub(super) fn other(
    right: &Expr,
    item: usize,
    resolver: &Resolver,
    positions: super::super::super::value::PlaceholderPositions<'_>,
) -> Option<Sourced> {
    let other = unwrap_expr(right);
    if let Some(array) = constructor(other) {
        return finite_array(&array.elem, item, resolver, positions);
    }
    // Retain stored-array syntax without crediting its owning row's bound.
    resolver.stored_source(other).map(|mut sourced| {
        if !matches!(sourced.source, SqlPinSource::Value)
            || !sourced.reads.is_empty()
            || !caller_array(other)
        {
            let items = match sourced.source {
                SqlPinSource::Items(items) => items,
                _ => Vec::new(),
            };
            sourced.source = SqlPinSource::StoredArray(items);
        }
        sourced
    })
}
