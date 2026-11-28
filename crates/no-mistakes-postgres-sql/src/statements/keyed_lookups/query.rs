mod ctes;
mod limits;
mod outputs;

use super::conditions::cannot_match;
use super::select_exprs::walk_select;
use super::{scope::ScopeStack, subquery, walk_nested_queries, walk_set_expr, LookupHistory};
use crate::idents::visit_shallow_exprs;
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
pub(super) use limits::has_zero_limit;
use limits::{discard_exists_limit, literal_limit_value};
pub(super) use outputs::per_row_output;
use outputs::{grouping_needs_rows, plain_output, select_has_plain_outputs};
use sqlparser::ast::{Expr, GroupByExpr, LimitClause, OrderByKind, Query, Select, SetExpr};

pub(super) fn walk_query(
    query: &Query,
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    walk_query_ordered(query, &[], scopes, out);
}

pub(super) fn walk_query_ordered(
    query: &Query,
    enclosing_order: &[&Expr],
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) -> LookupHistory {
    let zero_limit = has_zero_limit(query.limit_clause.as_ref());
    ctes::walk_reachable(query, zero_limit, scopes, out);
    let local_order;
    let order = match query.order_by.as_ref().map(|order| &order.kind) {
        Some(OrderByKind::Expressions(exprs)) => {
            local_order = exprs
                .iter()
                .map(|item| &item.expr)
                .chain(enclosing_order.iter().copied())
                .collect::<Vec<_>>();
            local_order.as_slice()
        }
        _ => enclosing_order,
    };
    let mut seen = if zero_limit {
        LookupHistory::new()
    } else if let SetExpr::Values(values) = query.body.as_ref() {
        let row_limit = direct_values_limit(query, enclosing_order);
        super::walk_values(values, row_limit, order, scopes, out)
    } else {
        walk_set_expr(&query.body, order, scopes, out)
    };
    inspect_limits(query, scopes, &mut seen, out);
    seen
}

fn direct_values_limit(query: &Query, enclosing_order: &[&Expr]) -> usize {
    if query.order_by.is_some() || !enclosing_order.is_empty() {
        return usize::MAX;
    }
    let Some(LimitClause::LimitOffset {
        limit: Some(limit),
        offset: None,
        ..
    }) = &query.limit_clause
    else {
        return usize::MAX;
    };
    literal_limit_value(limit)
        .and_then(|count| usize::try_from(count).ok())
        .unwrap_or(usize::MAX)
}

pub(super) fn walk_exists(
    query: &Query,
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    if let Some(select) = query
        .body
        .as_select()
        .filter(|select| discard_exists_projection(query, select))
    {
        walk_select(select, &[], true, scopes, out);
    } else {
        walk_query(query, scopes, out);
    }
}

fn discard_exists_projection(query: &Query, select: &Select) -> bool {
    query.with.is_none()
        && has_plain_order(query)
        && discard_exists_limit(query.limit_clause.as_ref())
        && query.fetch.is_none()
        && query.locks.is_empty()
        && select_has_plain_outputs(select)
}

fn has_plain_order(query: &Query) -> bool {
    query.order_by.as_ref().is_none_or(|order| {
        matches!(&order.kind, OrderByKind::Expressions(items)
            if items.iter().all(|item| plain_output(&item.expr)))
    })
}

pub(super) fn query_produces_no_rows(query: &Query) -> bool {
    has_zero_limit(query.limit_clause.as_ref())
        || has_plain_order(query)
            && match query.body.as_ref() {
                SetExpr::Select(select) => select_produces_no_rows(select),
                SetExpr::Query(inner) => query_produces_no_rows(inner),
                _ => false,
            }
}

pub(super) fn select_produces_no_rows(select: &Select) -> bool {
    select.selection.as_ref().is_some_and(cannot_match)
        && (grouping_needs_rows(&select.group_by)
            || select.having.is_none()
                && matches!(&select.group_by, GroupByExpr::Expressions(keys, _) if keys.is_empty())
                && select
                    .projection
                    .iter()
                    .filter_map(super::select_item_expr)
                    .all(per_row_output))
}

fn inspect_limits(
    query: &Query,
    scopes: &mut ScopeStack,
    seen: &mut LookupHistory,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    if query.limit_clause.is_none() && query.fetch.is_none() {
        return;
    }
    let outer = scopes.snapshot();
    let targets = super::select_exprs::SelectTargets::default();
    let mut visit = |expr: &Expr| subquery::inspect_expr(expr, &outer, &targets, seen, out);
    visit_shallow_exprs(&query.limit_clause, &mut visit);
    visit_shallow_exprs(&query.fetch, &mut visit);
    walk_nested_queries(&query.limit_clause, scopes, out);
    walk_nested_queries(&query.fetch, scopes, out);
}
