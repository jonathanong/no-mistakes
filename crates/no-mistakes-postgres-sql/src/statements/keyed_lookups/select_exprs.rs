mod identity;

use super::grouping;
use super::join::register_table_with_joins;
use super::scope::ScopeStack;
use super::subquery::inspect_expr;
use super::windows::referenced_expressions;
use super::{select_item_expr, walk_nested_queries, LookupHistory};
use crate::fx::FxHashSet;
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
pub(super) use identity::target_identity;
use identity::{normalize_sort_options, normalize_target};
use sqlparser::ast::{Distinct, Expr, Select, SelectItem, WindowSpec};
use std::borrow::Cow;
use std::cell::{OnceCell, RefCell};

#[derive(Default)]
pub(super) struct SelectTargets<'a> {
    projection: &'a [SelectItem],
    grouping: Option<&'a grouping::GroupReuse>,
    keys: OnceCell<RefCell<FxHashSet<Expr>>>,
    windows: RefCell<FxHashSet<WindowSpec>>,
}

impl<'a> SelectTargets<'a> {
    fn new(projection: &'a [SelectItem], grouping: &'a grouping::GroupReuse) -> Self {
        Self {
            projection,
            grouping: Some(grouping),
            ..Self::default()
        }
    }

    pub(super) fn filter_grouped<'expr>(&self, expr: &'expr Expr) -> Cow<'expr, Expr> {
        self.grouping
            .map_or(Cow::Borrowed(expr), |grouping| grouping.filter(expr))
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
    let outer = scopes.snapshot();
    let group_by = grouping::targets(&select.group_by);
    let reuse = grouping::GroupReuse::new(&group_by);
    let targets = SelectTargets::new(&select.projection, &reuse);
    for expr in expressions(select, order_by, &targets, &group_by) {
        inspect_expr(&expr, &outer, &targets, &mut seen, out);
        walk_nested_queries(expr.as_ref(), scopes, out);
    }
    scopes.pop_frame();
    seen
}

enum Inspection {
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
) -> impl Iterator<Item = Cow<'a, Expr>> {
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
                .map(|expr| (expr, Inspection::Independent)),
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
            referenced_expressions(select, distinct_on, order_by).map(|(expr, can_reuse)| {
                (
                    expr,
                    if can_reuse {
                        Inspection::Reusable
                    } else {
                        Inspection::Independent
                    },
                )
            }),
        )
        .chain(order_by.iter().copied().map(reusable))
        .filter_map(move |(expr, inspection)| match inspection {
            Inspection::Independent => Some(Cow::Borrowed(expr)),
            Inspection::Projection => Some(select_targets.filter_grouped(expr)),
            Inspection::Reusable => {
                (!select_targets.reuses(expr)).then(|| select_targets.filter_grouped(expr))
            }
            Inspection::Grouping => {
                if !seen_grouping.insert(target_identity(expr)) {
                    return None;
                }
                select_targets.reuses(expr);
                Some(Cow::Borrowed(expr))
            }
        })
}
