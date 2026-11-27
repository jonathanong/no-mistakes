use super::select_item_expr;
use super::windows::referenced_expressions;
use crate::fx::FxHashSet;
use crate::idents::unwrap_expr;
use sqlparser::ast::{Distinct, Expr, GroupByExpr, Select};

pub(super) fn inspectable<'a>(
    select: &'a Select,
    order_by: &'a [&'a Expr],
) -> impl Iterator<Item = &'a Expr> {
    let distinct_on = match &select.distinct {
        Some(Distinct::On(expressions)) => expressions.as_slice(),
        _ => &[],
    };
    let group_by = match &select.group_by {
        GroupByExpr::Expressions(expressions, _) => expressions.as_slice(),
        GroupByExpr::All(_) => &[],
    };

    let projection = select.projection.iter().filter_map(select_item_expr);
    let mut targets = FxHashSet::default();
    if !(distinct_on.is_empty()
        && group_by.is_empty()
        && select.named_window.is_empty()
        && order_by.is_empty())
    {
        targets.extend(projection.clone().map(unwrap_expr));
    }

    let clause_target = |expr| (expr, true);
    let independent = |expr| (expr, false);
    distinct_on
        .iter()
        .map(clause_target)
        .chain(projection.map(independent))
        .chain(select.selection.iter().map(independent))
        .chain(group_by.iter().map(clause_target))
        .chain(select.having.iter().map(independent))
        .chain(referenced_expressions(select, order_by).map(clause_target))
        .chain(order_by.iter().copied().map(clause_target))
        .filter_map(move |(expr, can_reuse)| {
            (!can_reuse || targets.insert(unwrap_expr(expr))).then_some(expr)
        })
}
