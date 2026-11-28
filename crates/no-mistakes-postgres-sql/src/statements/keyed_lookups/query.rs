mod ctes;

use super::conditions::literal;
use super::select_exprs::walk_select;
use super::select_item_expr;
use super::{scope::ScopeStack, subquery, walk_nested_queries, walk_set_expr, LookupHistory};
use crate::idents::{unwrap_expr, visit_shallow_exprs};
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
use sqlparser::ast::{Distinct, Expr, GroupByExpr, LimitClause, OrderByKind, Query, Select, Value};

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
    ctes::walk_reachable(query, scopes, out);
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
    let mut seen = if has_zero_limit(query.limit_clause.as_ref()) {
        LookupHistory::new()
    } else {
        walk_set_expr(&query.body, order, scopes, out)
    };
    inspect_limits(query, scopes, &mut seen, out);
    seen
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
        && query.order_by.is_none()
        && query.limit_clause.is_none()
        && query.fetch.is_none()
        && query.locks.is_empty()
        && select.having.is_none()
        && matches!(
            select.distinct,
            None | Some(Distinct::All | Distinct::Distinct)
        )
        && matches!(&select.group_by, GroupByExpr::Expressions(exprs, modifiers)
            if exprs.is_empty() && modifiers.is_empty())
        && select
            .projection
            .iter()
            .filter_map(select_item_expr)
            .all(|expr| {
                matches!(
                    unwrap_expr(expr),
                    Expr::Identifier(_)
                        | Expr::CompoundIdentifier(_)
                        | Expr::Value(_)
                        | Expr::Subquery(_)
                )
            })
}

pub(super) fn has_zero_limit(limit: Option<&LimitClause>) -> bool {
    let Some(LimitClause::LimitOffset {
        limit: Some(count), ..
    }) = limit
    else {
        return false;
    };
    match literal(count) {
        Some(Value::Number(number, _)) => {
            number.starts_with('0') && number.bytes().all(|digit| matches!(digit, b'0' | b'_'))
        }
        Some(Value::SingleQuotedString(number) | Value::EscapedStringLiteral(number)) => {
            number.trim_ascii().parse::<i64>() == Ok(0)
        }
        _ => false,
    }
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
