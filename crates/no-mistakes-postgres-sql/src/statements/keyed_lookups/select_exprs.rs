mod identity;

use super::function_clauses::is_eager;
use super::grouping;
use super::join::register_table_with_joins;
use super::scope::{ScopeSnapshot, ScopeStack};
use super::subquery::inspect_expr;
use super::windows::referenced_expressions;
use super::{select_item_expr, walk_nested_queries, LookupHistory};
use crate::fx::{FxHashMap, FxHashSet};
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
pub(super) use identity::target_identity;
use identity::{normalize_sort_options, normalize_target};
use sqlparser::ast::{Distinct, Expr, Function, Select, SelectItem, WindowSpec};
use std::borrow::Cow;
use std::cell::{OnceCell, RefCell};

#[derive(Default)]
pub(super) struct SelectTargets<'a> {
    projection: &'a [SelectItem],
    grouping: Option<&'a grouping::GroupReuse>,
    before_grouping: Option<&'a ScopeSnapshot>,
    keys: OnceCell<RefCell<FxHashSet<Expr>>>,
    windows: RefCell<FxHashSet<WindowSpec>>,
    filtered: RefCell<FxHashMap<Expr, LookupHistory>>,
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

    pub(super) fn take_filter_history(&self, filter: &Expr) -> LookupHistory {
        self.filtered
            .borrow_mut()
            .remove(filter)
            .unwrap_or_default()
    }

    pub(super) fn save_filter_history(&self, filter: Expr, history: LookupHistory) {
        self.filtered.borrow_mut().insert(filter, history);
    }

    pub(super) fn reuses_window(&self, spec: &WindowSpec) -> bool {
        if spec.window_frame.is_none() {
            return false;
        }
        let mut key = spec.clone();
        normalize_sort_options(&mut key.order_by);
        normalize_target(&mut key);
        !self.windows.borrow_mut().insert(key)
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
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) -> LookupHistory {
    scopes.push_frame();
    let mut seen = LookupHistory::new();
    for table in &select.from {
        register_table_with_joins(table, scopes, &mut seen, out);
    }
    let before_grouping = scopes.snapshot();
    let after_grouping = grouping::can_omit_keys(&select.group_by).then(|| {
        scopes.refresh_bindings(0..scopes.binding_count());
        scopes.snapshot()
    });
    let group_by = grouping::targets(&select.group_by, &select.projection);
    let reuse = grouping::GroupReuse::new(&group_by);
    let targets = SelectTargets {
        projection: &select.projection,
        grouping: Some(&reuse),
        before_grouping: after_grouping.as_ref().map(|_| &before_grouping),
        ..SelectTargets::default()
    };
    for (expr, runs_before_grouping) in expressions(select, order_by, &targets, &group_by) {
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
        .chain(
            referenced_expressions(select, distinct_on, order_by, select_targets).map(
                |(expr, can_reuse)| {
                    (
                        expr,
                        if can_reuse {
                            Inspection::Reusable
                        } else {
                            Inspection::Independent
                        },
                    )
                },
            ),
        )
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
