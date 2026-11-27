mod identity;

use super::select_item_expr;
use super::windows::referenced_expressions;
use crate::fx::FxHashSet;
use identity::normalize_target;
pub(super) use identity::target_identity;
use sqlparser::ast::{Distinct, Expr, Select, SelectItem, WindowSpec};
use std::cell::{OnceCell, RefCell};

#[derive(Default)]
pub(super) struct SelectTargets<'a> {
    projection: &'a [SelectItem],
    keys: OnceCell<RefCell<FxHashSet<Expr>>>,
    windows: RefCell<FxHashSet<WindowSpec>>,
}

impl<'a> SelectTargets<'a> {
    pub(super) fn new(projection: &'a [SelectItem]) -> Self {
        Self {
            projection,
            ..Self::default()
        }
    }

    pub(super) fn reuses_window(&self, spec: &WindowSpec) -> bool {
        if spec.window_frame.is_none() {
            return false;
        }
        let mut key = spec.clone();
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

pub(super) fn inspectable<'a>(
    select: &'a Select,
    order_by: &'a [&'a Expr],
    select_targets: &'a SelectTargets<'a>,
    grouping_targets: &'a [&'a Expr],
    having: Option<&'a Expr>,
) -> impl Iterator<Item = &'a Expr> {
    let distinct_on = match &select.distinct {
        Some(Distinct::On(expressions)) => expressions.as_slice(),
        _ => &[],
    };
    let projection = select.projection.iter().filter_map(select_item_expr);

    let clause_target = |expr| (expr, true);
    let independent = |expr| (expr, false);
    distinct_on
        .iter()
        .map(clause_target)
        .chain(projection.map(independent))
        .chain(select.selection.iter().map(independent))
        .chain(grouping_targets.iter().copied().map(clause_target))
        .chain(having.into_iter().map(independent))
        .chain(referenced_expressions(select, distinct_on, order_by))
        .chain(order_by.iter().copied().map(clause_target))
        .filter_map(move |(expr, can_reuse)| {
            (!can_reuse || !select_targets.reuses(expr)).then_some(expr)
        })
}
