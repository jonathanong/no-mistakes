mod ctes;

use super::conditions::{cannot_match, decimal_integer, literal};
use super::select_exprs::walk_select;
use super::select_item_expr;
use super::{scope::ScopeStack, subquery, walk_nested_queries, walk_set_expr, LookupHistory};
use crate::idents::{unwrap_expr, visit_shallow_exprs};
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
use sqlparser::ast::{
    CastKind, DataType, Distinct, Expr, GroupByExpr, LimitClause, OrderByKind, Query, Select,
    SetExpr, Value,
};

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
    select.selection.as_ref().is_some_and(cannot_match) && select_has_plain_outputs(select)
}

fn select_has_plain_outputs(select: &Select) -> bool {
    select.having.is_none()
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
            .all(plain_output)
}

pub(super) fn plain_output(expr: &Expr) -> bool {
    matches!(
        unwrap_expr(expr),
        Expr::Identifier(_) | Expr::CompoundIdentifier(_) | Expr::Value(_) | Expr::Subquery(_)
    )
}

fn discard_exists_limit(limit: Option<&LimitClause>) -> bool {
    match limit {
        None => true,
        Some(LimitClause::LimitOffset {
            limit,
            offset: None,
            ..
        }) => limit.as_ref().is_none_or(|count| {
            matches!(limit_literal(count), Some(Value::Null))
                || literal_limit_value(count).is_some_and(|value| value > 0)
        }),
        _ => false,
    }
}

pub(super) fn has_zero_limit(limit: Option<&LimitClause>) -> bool {
    let Some(LimitClause::LimitOffset {
        limit: Some(count), ..
    }) = limit
    else {
        return false;
    };
    literal_limit_value(count) == Some(0)
}

fn literal_limit_value(count: &Expr) -> Option<i64> {
    match limit_literal(count)? {
        Value::Number(number, _) => {
            decimal_integer(number).or_else(|| decimal_zero(number).then_some(0))
        }
        Value::SingleQuotedString(number) | Value::EscapedStringLiteral(number) => {
            number.trim_ascii().parse().ok()
        }
        _ => None,
    }
}

fn limit_literal(count: &Expr) -> Option<&Value> {
    let count = match unwrap_expr(count) {
        Expr::Cast {
            kind: CastKind::Cast | CastKind::DoubleColon,
            expr,
            data_type: DataType::BigInt(None),
            format: None,
            ..
        } => expr.as_ref(),
        count => count,
    };
    literal(count)
}

fn decimal_zero(number: &str) -> bool {
    let mantissa = number.split(['e', 'E']).next().unwrap_or_default();
    let mantissa = mantissa.trim_start_matches(['+', '-']);
    mantissa.contains('0')
        && mantissa
            .bytes()
            .all(|digit| matches!(digit, b'0' | b'.' | b'_'))
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
