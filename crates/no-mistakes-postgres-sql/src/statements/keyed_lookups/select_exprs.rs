use super::select_item_expr;
use super::windows::referenced_expressions;
use crate::fx::FxHashSet;
use crate::idents::{ident_key, unwrap_expr};
use sqlparser::ast::{Distinct, Expr, GroupByExpr, Ident, Select, VisitMut, VisitorMut};
use std::ops::ControlFlow;

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
        targets.extend(projection.clone().map(target_identity));
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
            (!can_reuse || targets.insert(target_identity(expr))).then_some(expr)
        })
}

fn target_identity(expr: &Expr) -> Expr {
    struct Identifiers;
    impl VisitorMut for Identifiers {
        type Break = ();

        fn pre_visit_ident(&mut self, ident: &mut Ident) -> ControlFlow<()> {
            ident.value = ident_key(ident);
            ident.quote_style = None;
            ControlFlow::Continue(())
        }
    }

    let mut key = unwrap_expr(expr).clone();
    let _ = key.visit(&mut Identifiers);
    key
}
