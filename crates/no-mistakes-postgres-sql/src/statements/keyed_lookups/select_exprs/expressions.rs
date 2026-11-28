use super::{select_item_expr, target_identity, SelectTargets, WindowPlan};
use crate::fx::FxHashSet;
use sqlparser::ast::{Distinct, Expr, Select};
use std::borrow::Cow;

#[derive(Clone, Copy)]
pub(super) enum Inspection {
    Selection,
    Independent,
    Projection,
    Grouping,
    Reusable,
}

impl Inspection {
    pub(super) fn before_grouping(self) -> bool {
        matches!(self, Self::Selection | Self::Grouping)
    }

    pub(super) fn has_row_filter(self) -> bool {
        !matches!(self, Self::Selection | Self::Independent)
    }
}

pub(super) fn expressions<'a>(
    select: &'a Select,
    order_by: &'a [&'a Expr],
    select_targets: &'a SelectTargets<'a>,
    grouping_targets: &'a [&'a Expr],
    windows: &'a WindowPlan,
) -> impl Iterator<Item = (Cow<'a, Expr>, Inspection)> {
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
                .map(|expr| (expr, Inspection::Selection)),
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
            Inspection::Selection | Inspection::Independent => {
                Some((Cow::Borrowed(expr), inspection))
            }
            Inspection::Projection => Some((select_targets.filter_grouped(expr), inspection)),
            Inspection::Reusable => (!select_targets.reuses(expr))
                .then(|| (select_targets.filter_grouped(expr), inspection)),
            Inspection::Grouping => {
                if !seen_grouping.insert(target_identity(expr)) {
                    return None;
                }
                select_targets.reuses(expr);
                Some((Cow::Borrowed(expr), inspection))
            }
        })
}
