use super::{work, Relations};
use crate::statements::limit::is_limited_with_columns;
use crate::statements::value::PlaceholderPositions;
use sqlparser::ast::{Expr, Ident, LimitClause, Query, SetExpr, TableFactor};
use std::cell::OnceCell;

pub(in crate::statements::bounds) fn examine(
    query: &Query,
    scope: &super::super::Scope,
    positions: PlaceholderPositions<'_>,
) -> (bool, Option<(String, String)>) {
    let visible = OnceCell::new();
    let limited = is_limited_with_columns(query, positions, &|expr| {
        prepare(query, scope, positions, &visible).contains(expr)
    });
    let catalog = catalog_site(query)
        .and_then(|parts| prepare(query, scope, positions, &visible).catalog_column(parts));
    (limited, catalog)
}

fn catalog_site(query: &Query) -> Option<&[Ident]> {
    if let Some(fetch) = &query.fetch {
        if fetch.with_ties || fetch.percent {
            return None;
        }
    }
    limit_column(query)
}

fn prepare<'a>(
    query: &'a Query,
    scope: &'a super::super::Scope,
    positions: PlaceholderPositions<'a>,
    visible: &'a OnceCell<Relations>,
) -> &'a Relations {
    visible.get_or_init(|| {
        work::visible_map_build();
        let mut visible = scope.limit_columns.clone();
        if let SetExpr::Select(select) = &*query.body {
            for table in &select.from {
                shadow(&mut visible, &table.relation, positions);
                for join in &table.joins {
                    shadow(&mut visible, &join.relation, positions);
                }
            }
        }
        visible
    })
}

fn limit_column(query: &Query) -> Option<&[Ident]> {
    let expr = if let Some(fetch) = &query.fetch {
        fetch.quantity.as_ref()?
    } else {
        match &query.limit_clause {
            Some(LimitClause::LimitOffset {
                limit: Some(limit), ..
            })
            | Some(LimitClause::OffsetCommaLimit { limit, .. }) => limit,
            _ => return None,
        }
    };
    column_root(expr)
}

fn column_root(expr: &Expr) -> Option<&[Ident]> {
    match expr {
        Expr::Nested(inner) | Expr::Cast { expr: inner, .. } => column_root(inner),
        Expr::CompoundIdentifier(parts) => Some(parts),
        _ => None,
    }
}

fn shadow(visible: &mut Relations, factor: &TableFactor, positions: PlaceholderPositions<'_>) {
    let mut local = Relations::default();
    local.add(factor, positions);
    visible.opaque = true;
    for name in local.names.into_keys() {
        visible.names.insert(
            name,
            super::Columns::Named(std::collections::BTreeMap::new()),
        );
    }
}
