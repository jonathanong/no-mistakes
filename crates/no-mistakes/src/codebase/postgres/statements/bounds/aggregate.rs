use super::functions::{
    data_backed_projection, is_aggregate, projection_can_expand, projection_traversal_boundary,
};
use crate::codebase::postgres::idents::visit_child_exprs;
use sqlparser::ast::{
    Expr, Function, GroupByExpr, OrderByKind, Query, Select, SelectItem, SetExpr,
};

/// An aggregate with no `GROUP BY` returns exactly one row, unless a set-returning function in
/// the select list expands it. HAVING itself introduces implicit single-group grouping.
pub(super) fn pure_aggregate(select: &Select) -> bool {
    one_group(select)
        && (projected(select)
            .iter()
            .any(|expr| contains_plain_aggregate(expr))
            || select.having.is_some())
}

/// Whether a SELECT has the implicit group that still exists when its WHERE rejects all rows.
pub(super) fn has_implicit_group(query: &Query, select: &Select) -> bool {
    if has_empty_grouping_set(select) {
        return true;
    }
    if !ungrouped(select) {
        return false;
    }
    if select.having.is_some()
        || projected(select)
            .iter()
            .any(|expr| contains_plain_aggregate(expr))
        // Inspect every PostgreSQL named-window definition. A chained window spec stores its
        // parent in `WindowSpec::window_name`, so its aggregate is found on the parent definition.
        || select.named_window.iter().any(|window| match &window.1 {
            sqlparser::ast::NamedWindowExpr::WindowSpec(spec) => {
                window_spec_has_plain_aggregate(spec)
            }
            sqlparser::ast::NamedWindowExpr::NamedWindow(_) => false,
        })
    {
        return true;
    }
    let Some(order) = &query.order_by else {
        return false;
    };
    let OrderByKind::Expressions(expressions) = &order.kind else {
        return false;
    };
    expressions
        .iter()
        .any(|expression| contains_plain_aggregate(&expression.expr))
}

/// An expanding projection backed by data prevents a FROM-free SELECT from proving a
/// bounded source for a relation joined to it.
pub(super) fn expands_from_data(select: &Select) -> bool {
    projected(select)
        .iter()
        .any(|expr| contains_projection_call(expr, &data_backed_projection))
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

/// Ungrouped, and with nothing in the select list that expands one row into many.
fn one_group(select: &Select) -> bool {
    ungrouped(select)
        && !projected(select)
            .iter()
            .any(|expr| contains_projection_call(expr, &projection_can_expand))
}

fn ungrouped(select: &Select) -> bool {
    matches!(
        &select.group_by,
        GroupByExpr::Expressions(expressions, modifiers)
            if expressions.is_empty() && modifiers.is_empty()
    )
}

/// An empty grouping set emits a group even when WHERE leaves no input rows.
fn has_empty_grouping_set(select: &Select) -> bool {
    fn allows_empty(expr: &Expr) -> bool {
        match expr {
            Expr::GroupingSets(groups) => groups.iter().any(|group| group.iter().all(allows_empty)),
            Expr::Rollup(_) | Expr::Cube(_) => true,
            Expr::Tuple(items) => items.iter().all(allows_empty),
            // sqlparser represents grouping constructs nested inside GROUPING SETS as calls.
            Expr::Function(function) if function.name.0.len() == 1 => {
                function.name.0[0].as_ident().is_some_and(|ident| {
                    ident.quote_style.is_none()
                        && matches!(ident.value.to_ascii_lowercase().as_str(), "rollup" | "cube")
                })
            }
            _ => false,
        }
    }
    matches!(
        &select.group_by,
        GroupByExpr::Expressions(expressions, modifiers)
            if modifiers.is_empty()
                && !expressions.is_empty()
                && expressions.iter().all(allows_empty)
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

fn is_plain_aggregate(function: &Function) -> bool {
    function.over.is_none() && is_aggregate(&function.name)
}

fn contains_plain_aggregate(expr: &Expr) -> bool {
    if let Expr::Function(function) = expr {
        if is_plain_aggregate(function) {
            return true;
        }
        if let Some(sqlparser::ast::WindowType::WindowSpec(spec)) = &function.over {
            if window_spec_has_plain_aggregate(spec) {
                return true;
            }
        }
    }
    let mut found = false;
    visit_child_exprs(expr, &mut |child| {
        found = found || contains_plain_aggregate(child)
    });
    found
}

fn window_spec_has_plain_aggregate(spec: &sqlparser::ast::WindowSpec) -> bool {
    spec.partition_by.iter().any(contains_plain_aggregate)
        || spec
            .order_by
            .iter()
            .any(|expression| contains_plain_aggregate(&expression.expr))
}

/// Projection expansion stops at functions whose PostgreSQL syntax guarantees scalar output.
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
