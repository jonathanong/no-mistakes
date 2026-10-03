use super::aggregate::{orders_by_aggregate, pure_aggregate};
use super::{items, start, Scope};
use crate::codebase::postgres::idents::ident_key;
use crate::codebase::postgres::statements::limit::is_limited;
use crate::codebase::postgres::statements::{
    SqlBoundItem, SqlBoundItemKind, SqlBoundQuery, SqlPinSource,
};
use sqlparser::ast::{Query, SetExpr, SetOperator, SetQuantifier, Spanned, Statement};
use std::collections::BTreeMap;

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
            // before twice grows exponentially. Compact oversized bounds conservatively,
            // retaining the distinct uncapped relations instead of hiding their reads.
            if size(&bound) > MAX_BOUND_ITEMS {
                compact(&bound, start(cte.query.span()))
            } else {
                bound
            }
        };
        let columns = if cte.alias.columns.is_empty() {
            super::pins::projection_columns(&cte.query)
        } else {
            Some(
                cte.alias
                    .columns
                    .iter()
                    .map(|column| ident_key(&column.name))
                    .collect(),
            )
        };
        scope.columns.insert(name.clone(), columns);
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

/// Keep a fail-closed summary of oversized CTEs. Capped subqueries contribute no
/// unbounded reads; an opaque item prevents this summary from sizing a later join.
fn compact(bound: &SqlBoundQuery, at: (usize, usize)) -> SqlBoundQuery {
    fn tables(query: &SqlBoundQuery, found: &mut BTreeMap<String, (usize, usize)>) {
        if query.capped {
            return;
        }
        for item in &query.items {
            match &item.kind {
                SqlBoundItemKind::Table(name) => {
                    found
                        .entry(name.clone())
                        .or_insert((item.line, item.column));
                }
                SqlBoundItemKind::Query(inner) => tables(inner, found),
                _ => {}
            }
        }
    }
    let mut found = BTreeMap::new();
    tables(bound, &mut found);
    let mut items: Vec<_> = found
        .into_iter()
        .map(|(name, at)| SqlBoundItem::new(SqlBoundItemKind::Table(name), None, at))
        .collect();
    items.push(items::opaque(at));
    SqlBoundQuery {
        capped: bound.capped,
        items,
    }
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
    bound.capped |=
        (is_limited(query) && !requires_complete_arms(&query.body)) || orders_by_aggregate(query);
    bound
}

/// Non-streaming set operations must inspect their input arms before an outer LIMIT can
/// produce the final rows; UNION ALL can stop as soon as that limit is satisfied.
fn requires_complete_arms(set: &SetExpr) -> bool {
    match set {
        SetExpr::SetOperation {
            op,
            set_quantifier,
            left,
            right,
        } => {
            *op != SetOperator::Union
                || *set_quantifier != SetQuantifier::All
                || requires_complete_arms(left)
                || requires_complete_arms(right)
        }
        SetExpr::Query(query) => requires_complete_arms(&query.body),
        _ => false,
    }
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
