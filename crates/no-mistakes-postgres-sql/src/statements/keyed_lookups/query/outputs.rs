use super::super::expr_norm::function::pure_name;
use super::super::select_item_expr;
use crate::idents::unwrap_expr;
use sqlparser::ast::{visit_expressions, Distinct, Expr, GroupByExpr, Select};
use std::ops::ControlFlow;

pub(super) fn select_has_plain_outputs(select: &Select) -> bool {
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

pub(in crate::statements::keyed_lookups) fn plain_output(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Identifier(_) | Expr::CompoundIdentifier(_) | Expr::Value(_) => true,
        Expr::Subquery(query) => visit_expressions(query, possible_outer_aggregate).is_continue(),
        _ => false,
    }
}

fn possible_outer_aggregate(expr: &Expr) -> ControlFlow<()> {
    let Expr::Function(function) = expr else {
        return ControlFlow::Continue(());
    };
    if function.over.is_some() || pure_name(function).is_some() {
        return ControlFlow::Continue(());
    }
    visit_expressions(function, |input| match input {
        Expr::Identifier(_) | Expr::CompoundIdentifier(_) => ControlFlow::Break(()),
        _ => ControlFlow::Continue(()),
    })
}
