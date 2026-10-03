use super::aggregate::{orders_by_aggregate, pure_aggregate};
use super::{items, start, Scope};
use crate::codebase::postgres::idents::ident_key;
use crate::codebase::postgres::statements::limit::is_limited;
use crate::codebase::postgres::statements::{
    SqlBoundItem, SqlBoundItemKind, SqlBoundQuery, SqlPinSource,
};
use sqlparser::ast::{Query, SetExpr, Spanned, Statement};

pub(super) fn bound_query(query: &Query, scope: &Scope) -> SqlBoundQuery {
    bound_body(query, &with_scope(query, scope))
}

/// The scope after this query's own `WITH` clause.
pub(super) fn with_scope(query: &Query, outer: &Scope) -> Scope {
    let mut scope = outer.clone();
    let Some(with) = &query.with else {
        return scope;
    };
    for cte in &with.cte_tables {
        let name = ident_key(&cte.alias.name);
        let bound = if modifying_statement(&cte.query).is_some() {
            // `RETURNING` yields one row per modified row, which nothing in the text sizes: it
            // bounds nothing pinned to it. The statement inside is judged on its own.
            opaque(start(cte.query.span()))
        } else {
            let mut inner = scope.clone();
            if with.recursive {
                // A recursive reference is whatever the recursion has produced so far: it
                // bounds nothing joined to it.
                inner.insert(name.clone(), opaque(start(cte.query.span())));
            }
            let bound = bound_query(&cte.query, &inner);
            // Each reference clones the CTE's bound, so a chain of CTEs that each read the one
            // before twice grows exponentially: past a generous size it is opaque instead.
            if size(&bound) > MAX_BOUND_ITEMS {
                opaque(start(cte.query.span()))
            } else {
                bound
            }
        };
        scope.insert(name, bound);
    }
    scope
}

const MAX_BOUND_ITEMS: usize = 2048;

/// The number of items in a bound, nested queries and subquery pins included.
fn size(query: &SqlBoundQuery) -> usize {
    query
        .items
        .iter()
        .map(|item| {
            let inner = match &item.kind {
                SqlBoundItemKind::Query(inner) => size(inner),
                _ => 0,
            };
            let pins: usize = item
                .pins
                .iter()
                .map(|pin| match &pin.source {
                    SqlPinSource::Query(inner) => size(inner),
                    _ => 0,
                })
                .sum();
            1 + inner + pins
        })
        .sum()
}

/// The `INSERT` / `UPDATE` / `DELETE` / `MERGE` inside a data-modifying CTE.
pub(super) fn modifying_statement(query: &Query) -> Option<&Statement> {
    match &*query.body {
        SetExpr::Update(statement)
        | SetExpr::Delete(statement)
        | SetExpr::Insert(statement)
        | SetExpr::Merge(statement) => Some(statement),
        _ => None,
    }
}

/// The body of `query` under `scope`, which already holds its CTEs.
pub(super) fn bound_body(query: &Query, scope: &Scope) -> SqlBoundQuery {
    let mut bound = set_bound(&query.body, scope);
    bound.capped |= is_limited(query) || orders_by_aggregate(query);
    bound
}

fn set_bound(set: &SetExpr, scope: &Scope) -> SqlBoundQuery {
    match set {
        SetExpr::Select(select) => SqlBoundQuery {
            capped: pure_aggregate(select),
            items: items::from_select(select, scope),
        },
        SetExpr::Query(query) => bound_query(query, scope),
        // A set operation returns the rows of both arms, so both must be bounded.
        SetExpr::SetOperation { left, right, .. } => SqlBoundQuery {
            capped: false,
            items: vec![arm(left, scope), arm(right, scope)],
        },
        SetExpr::Table(table) => super::table::bound(table, scope, start(set.span())),
        _ => sized_by_itself(start(set.span())),
    }
}

fn arm(set: &SetExpr, scope: &Scope) -> SqlBoundItem {
    let kind = SqlBoundItemKind::Query(set_bound(set, scope));
    SqlBoundItem::new(kind, None, start(set.span()))
}

/// A body whose size nothing in the statement text decides: it adds no unbounded relation.
pub(super) fn sized_by_itself(at: (usize, usize)) -> SqlBoundQuery {
    SqlBoundQuery {
        capped: false,
        items: vec![items::other(at)],
    }
}

/// A body whose rows nothing proves bounded, and which is never reported either.
fn opaque(at: (usize, usize)) -> SqlBoundQuery {
    SqlBoundQuery {
        capped: false,
        items: vec![items::opaque(at)],
    }
}
