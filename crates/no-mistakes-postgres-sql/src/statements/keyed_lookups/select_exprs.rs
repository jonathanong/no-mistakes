pub(super) mod identity;

use super::conditions::cannot_match;
use super::function_clauses::is_eager;
use super::grouping;
use super::join::register_table_with_joins;
use super::query::{plain_output, select_has_plain_outputs};
use super::scope::{ScopeSnapshot, ScopeStack};
use super::subquery::inspect_expr;
use super::windows::WindowPlan;
use super::{inspect_exprs, select_item_expr, walk_nested_queries, LookupHistory};
use crate::fx::FxHashSet;
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
pub(super) use identity::target_identity;
use sqlparser::ast::{Distinct, Expr, Function, Select, SelectItem, WindowSpec};
use std::borrow::Cow;
use std::cell::{OnceCell, RefCell};

#[derive(Default)]
pub(super) struct SelectTargets<'a> {
    projection: &'a [SelectItem],
    grouping: Option<&'a grouping::GroupReuse>,
    before_grouping: Option<&'a ScopeSnapshot>,
    keys: OnceCell<RefCell<FxHashSet<Expr>>>,
    windows: Option<&'a WindowPlan>,
}

impl SelectTargets<'_> {
    pub(super) fn function_input_scope<'scope>(
        &'scope self,
        function: &Function,
        outer: &'scope ScopeSnapshot,
    ) -> &'scope ScopeSnapshot {
        self.before_grouping
            .filter(|_| function.over.is_none() && is_eager(function))
            .unwrap_or(outer)
    }

    pub(super) fn filter_grouped<'expr>(&self, expr: &'expr Expr) -> Cow<'expr, Expr> {
        self.grouping
            .map_or(Cow::Borrowed(expr), |grouping| grouping.filter(expr))
    }

    pub(super) fn skips_window_call(&self, expr: &Expr) -> bool {
        self.windows.is_some_and(|windows| windows.skips_call(expr))
    }

    pub(super) fn take_inline_frame(&self, spec: &WindowSpec) -> Option<bool> {
        self.windows
            .map_or(Some(true), |windows| windows.take_inline_frame(spec))
    }

    pub(super) fn reuses(&self, expr: &Expr) -> bool {
        !self
            .keys
            .get_or_init(|| {
                RefCell::new(
                    self.projection
                        .iter()
                        .filter_map(select_item_expr)
                        .map(target_identity)
                        .collect(),
                )
            })
            .borrow_mut()
            .insert(target_identity(expr))
    }
}

pub(super) fn walk_select(
    select: &Select,
    order_by: &[&Expr],
    discard_projection: bool,
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) -> LookupHistory {
    if select.selection.as_ref().is_some_and(cannot_match)
        && select_has_plain_outputs(select)
        && order_by.iter().all(|expr| plain_output(expr))
    {
        return LookupHistory::new();
    }
    scopes.push_frame();
    let mut seen = LookupHistory::new();
    for table in &select.from {
        register_table_with_joins(table, scopes, &mut seen, out);
    }
    if discard_projection {
        inspect_exprs(select.selection.iter(), scopes, &mut seen, out);
        scopes.pop_frame();
        return seen;
    }
    let before_grouping = scopes.snapshot();
    let after_grouping = grouping::can_omit_keys(&select.group_by).then(|| {
        scopes.refresh_bindings(0..scopes.binding_count());
        scopes.snapshot()
    });
    let group_by = grouping::GroupingTargets::new(select);
    let reuse = grouping::GroupReuse::new(&group_by);
    let windows = WindowPlan::new(select, order_by);
    let targets = SelectTargets {
        projection: &select.projection,
        grouping: Some(&reuse),
        before_grouping: after_grouping.as_ref().map(|_| &before_grouping),
        windows: Some(&windows),
        ..SelectTargets::default()
    };
    for (expr, runs_before_grouping) in
        expressions(select, order_by, &targets, &group_by.all, &windows)
    {
        let outer = after_grouping
            .as_ref()
            .filter(|_| !runs_before_grouping)
            .unwrap_or(&before_grouping);
        inspect_expr(&expr, outer, &targets, &mut seen, out);
        walk_nested_queries(expr.as_ref(), scopes, out);
    }
    scopes.pop_frame();
    seen
}

enum Inspection {
    BeforeGrouping,
    Independent,
    Projection,
    Grouping,
    Reusable,
}

fn expressions<'a>(
    select: &'a Select,
    order_by: &'a [&'a Expr],
    select_targets: &'a SelectTargets<'a>,
    grouping_targets: &'a [&'a Expr],
    windows: &'a WindowPlan,
) -> impl Iterator<Item = (Cow<'a, Expr>, bool)> {
    let distinct_on = match &select.distinct {
        Some(Distinct::On(expressions)) => expressions.as_slice(),
        _ => &[],
    };
    let mut seen_grouping = FxHashSet::default();
    let projection = select.projection.iter().filter_map(select_item_expr);
    let reusable = |expr| (expr, Inspection::Reusable);
    distinct_on
        .iter()
        .map(reusable)
        .chain(projection.map(|expr| (expr, Inspection::Projection)))
        .chain(
            select
                .selection
                .iter()
                .map(|expr| (expr, Inspection::BeforeGrouping)),
        )
        .chain(
            grouping_targets
                .iter()
                .map(|&expr| (expr, Inspection::Grouping)),
        )
        .chain(
            select
                .having
                .iter()
                .map(|expr| (expr, Inspection::Projection)),
        )
        .chain(windows.named_expressions(select).map(|(expr, can_reuse)| {
            (
                expr,
                if can_reuse {
                    Inspection::Reusable
                } else {
                    Inspection::Independent
                },
            )
        }))
        .chain(order_by.iter().copied().map(reusable))
        .filter_map(move |(expr, inspection)| match inspection {
            Inspection::BeforeGrouping => Some((Cow::Borrowed(expr), true)),
            Inspection::Independent => Some((Cow::Borrowed(expr), false)),
            Inspection::Projection => Some((select_targets.filter_grouped(expr), false)),
            Inspection::Reusable => {
                (!select_targets.reuses(expr)).then(|| (select_targets.filter_grouped(expr), false))
            }
            Inspection::Grouping => {
                if !seen_grouping.insert(target_identity(expr)) {
                    return None;
                }
                select_targets.reuses(expr);
                Some((Cow::Borrowed(expr), true))
            }
        })
}
