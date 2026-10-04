use super::functions::{
    data_backed_projection, projection_can_expand, projection_traversal_boundary,
};
use crate::codebase::postgres::idents::visit_child_exprs;
use sqlparser::ast::{Distinct, Expr, Function, OrderByKind, Query, Select, SelectItem, SetExpr};

mod implicit_group;
use implicit_group::contains_plain_aggregate;
pub(super) use implicit_group::has_implicit_group;

/// An aggregate with no `GROUP BY` returns exactly one row, unless a set-returning function in
/// the select list expands it. HAVING itself introduces implicit single-group grouping.
pub(super) fn pure_aggregate(select: &Select) -> bool {
    one_group(select)
        && (projected(select)
            .iter()
            .any(|expr| contains_plain_aggregate(expr))
            || select.having.is_some())
}

/// An expanding projection backed by data prevents a FROM-free SELECT from proving a
/// bounded source for a relation joined to it.
pub(super) fn expands_from_data(
    select: &Select,
    positions: super::super::value::PlaceholderPositions<'_>,
) -> bool {
    projected(select).iter().any(|expr| {
        contains_projection_call(expr, &|function| {
            data_backed_projection(function, positions)
        })
    })
}

/// An aggregate used only in `ORDER BY` (`SELECT 1 FROM t ORDER BY count(*)`) makes an ungrouped
/// query a single row as well.
pub(super) fn orders_by_aggregate(query: &Query) -> bool {
    let (SetExpr::Select(select), Some(order)) = (&*query.body, &query.order_by) else {
        return false;
    };
    let OrderByKind::Expressions(expressions) = &order.kind else {
        return false;
    };
    one_group(select)
        && !orders_can_expand(query)
        && expressions
            .iter()
            .any(|expression| contains_plain_aggregate(&expression.expr))
}

/// ORDER BY expressions can expand the same implicit group as SELECT-list SRFs.
pub(super) fn orders_can_expand(query: &Query) -> bool {
    let Some(order) = &query.order_by else {
        return false;
    };
    let OrderByKind::Expressions(expressions) = &order.kind else {
        return false;
    };
    expressions
        .iter()
        .any(|expression| contains_projection_call(&expression.expr, &projection_can_expand))
}

/// A predicate that removes the implicit group can cap a query whose ordering expands it.
pub(super) fn order_expansion_predicates_reject(query: &Query) -> Option<bool> {
    if !orders_can_expand(query) {
        return None;
    }
    let (select, nested_empty) = select_body(&query.body)?;
    Some(
        nested_empty
            || super::predicate::rejects_all(select.having.as_ref())
            || (!has_implicit_group(query, select)
                && super::predicate::rejects_all(select.selection.as_ref())),
    )
}

fn select_body(set: &SetExpr) -> Option<(&Select, bool)> {
    match set {
        SetExpr::Select(select) => Some((select, false)),
        SetExpr::Query(query) => {
            let (select, empty) = select_body(&query.body)?;
            // An outer SRF cannot expand a row that an inner zero limit removed.
            Some((select, empty || super::super::limit::is_zero_limited(query)))
        }
        _ => None,
    }
}

/// Ungrouped, and with nothing in the select list that expands one row into many.
fn one_group(select: &Select) -> bool {
    implicit_group::ungrouped(select)
        && !projected(select)
            .iter()
            .any(|expr| contains_projection_call(expr, &projection_can_expand))
        && (!distinct_on_expands(select) || super::predicate::rejects_all(select.having.as_ref()))
}

/// DISTINCT ON evaluates its expressions against the grouped rows, including implicit groups.
/// A set-returning call there can expand the single aggregate row just like a projected call.
fn distinct_on_expands(select: &Select) -> bool {
    matches!(
        &select.distinct,
        Some(Distinct::On(expressions))
            if expressions.iter().any(|expr| contains_projection_call(expr, &|function| {
                projection_can_expand(function)
            }))
    )
}

fn projected(select: &Select) -> Vec<&Expr> {
    select
        .projection
        .iter()
        .filter_map(|item| match item {
            SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => Some(expr),
            _ => None,
        })
        .collect()
}

/// Whether a call that `test` accepts occurs in `expr` (subqueries are not looked into).
fn contains_projection_call(expr: &Expr, test: &impl Fn(&Function) -> bool) -> bool {
    if let Expr::Function(function) = expr {
        if test(function) {
            return true;
        }
        if projection_traversal_boundary(function) {
            return false;
        }
    }
    let mut found = false;
    visit_child_exprs(expr, &mut |child| {
        found = found || contains_projection_call(child, test)
    });
    found
}
