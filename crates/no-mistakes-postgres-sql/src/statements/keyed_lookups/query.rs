use super::conditions::literal;
use super::{scope::ScopeStack, subquery, walk_nested_queries, walk_set_expr, LookupHistory};
use crate::idents::visit_shallow_exprs;
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
use sqlparser::ast::{Expr, LimitClause, OrderByKind, Query, Value};

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
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            walk_query(&cte.query, scopes, out);
        }
    }
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
