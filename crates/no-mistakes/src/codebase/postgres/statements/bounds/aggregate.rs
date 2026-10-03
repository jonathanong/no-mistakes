use super::functions::{is_aggregate, is_set_returning};
use crate::codebase::postgres::idents::visit_child_exprs;
use sqlparser::ast::{
    Expr, Function, GroupByExpr, OrderByKind, Query, Select, SelectItem, SetExpr,
};

/// An aggregate with no `GROUP BY` returns exactly one row, unless a set-returning function in
/// the select list expands it. The aggregate may sit in `HAVING` alone.
pub(super) fn pure_aggregate(select: &Select) -> bool {
    one_group(select)
        && (projected(select)
            .iter()
            .any(|expr| contains_call(expr, &is_plain_aggregate))
            || select
                .having
                .as_ref()
                .is_some_and(|having| contains_call(having, &is_plain_aggregate)))
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
        && expressions
            .iter()
            .any(|expression| contains_call(&expression.expr, &is_plain_aggregate))
}

/// Ungrouped, and with nothing in the select list that expands one row into many.
fn one_group(select: &Select) -> bool {
    let ungrouped = matches!(
        &select.group_by,
        GroupByExpr::Expressions(expressions, modifiers)
            if expressions.is_empty() && modifiers.is_empty()
    );
    ungrouped
        && !projected(select)
            .iter()
            .any(|expr| contains_call(expr, &|function| is_set_returning(&function.name)))
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

fn is_plain_aggregate(function: &Function) -> bool {
    function.over.is_none() && is_aggregate(&function.name)
}

/// Whether a call that `test` accepts occurs in `expr` (subqueries are not looked into).
fn contains_call(expr: &Expr, test: &impl Fn(&Function) -> bool) -> bool {
    if let Expr::Function(function) = expr {
        if test(function) {
            return true;
        }
    }
    let mut found = false;
    visit_child_exprs(expr, &mut |child| {
        found = found || contains_call(child, test)
    });
    found
}
