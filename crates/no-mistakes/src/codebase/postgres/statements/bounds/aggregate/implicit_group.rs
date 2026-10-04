use super::super::functions::is_aggregate;
use crate::codebase::postgres::idents::visit_child_exprs;
use sqlparser::ast::{Expr, Function, GroupByExpr, OrderByKind, Query, Select};

/// Whether a SELECT still emits an implicit or empty grouping set when WHERE removes every row.
pub(in crate::codebase::postgres::statements::bounds) fn has_implicit_group(
    query: &Query,
    select: &Select,
) -> bool {
    if has_empty_grouping_set(select) {
        return true;
    }
    if !ungrouped(select) {
        return false;
    }
    if select.having.is_some()
        || super::projected(select)
            .iter()
            .any(|expr| contains_plain_aggregate(expr))
        // A chained named window stores its parent in WindowSpec::window_name.
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

pub(super) fn ungrouped(select: &Select) -> bool {
    matches!(
        &select.group_by,
        GroupByExpr::Expressions(expressions, modifiers)
            if expressions.is_empty() && modifiers.is_empty()
    )
}

fn has_empty_grouping_set(select: &Select) -> bool {
    fn allows_empty(expr: &Expr) -> bool {
        match expr {
            Expr::GroupingSets(groups) => groups.iter().any(|group| group.iter().all(allows_empty)),
            Expr::Rollup(_) | Expr::Cube(_) => true,
            Expr::Tuple(items) => items.iter().all(allows_empty),
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

pub(super) fn contains_plain_aggregate(expr: &Expr) -> bool {
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

fn is_plain_aggregate(function: &Function) -> bool {
    function.over.is_none() && is_aggregate(&function.name)
}

fn window_spec_has_plain_aggregate(spec: &sqlparser::ast::WindowSpec) -> bool {
    spec.partition_by.iter().any(contains_plain_aggregate)
        || spec
            .order_by
            .iter()
            .any(|expression| contains_plain_aggregate(&expression.expr))
}
