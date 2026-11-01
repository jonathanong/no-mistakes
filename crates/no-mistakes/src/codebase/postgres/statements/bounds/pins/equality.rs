use super::resolver::{Resolver, Sourced};
use crate::codebase::postgres::statements::{SqlBoundPin, SqlPinSource};
use sqlparser::ast::Expr;

/// Pin either side of `left = right` that is a column of one item to the other side.
pub(super) fn equate(
    left: &Expr,
    right: &Expr,
    null_safe: bool,
    resolver: &Resolver,
    pin: &mut impl FnMut(&Expr, Option<Sourced>, bool),
) {
    if let Some((item, _)) = resolver.column(left) {
        pin(
            left,
            resolver
                .source(right, item)
                .or_else(|| resolver.correlated_source(right)),
            null_safe,
        );
    }
    if let Some((item, _)) = resolver.column(right) {
        pin(
            right,
            resolver
                .source(left, item)
                .or_else(|| resolver.correlated_source(left)),
            null_safe,
        );
    }
}

/// Remember the other column of a one-item equality so a later value pin can cross the join.
pub(super) fn note_peers(
    left: &Expr,
    right: &Expr,
    resolver: &Resolver,
    out: &mut [(usize, SqlBoundPin)],
) {
    let Some((left_item, left_column)) = resolver.column(left) else {
        return;
    };
    let Some((right_item, right_column)) = resolver.column(right) else {
        return;
    };
    if left_item == right_item {
        return;
    }
    for (item, pin) in out {
        let SqlPinSource::Items(sources) = &pin.source else {
            continue;
        };
        let [source] = sources.as_slice() else {
            continue;
        };
        if pin.peer_column.is_some() {
            continue;
        }
        if *item == left_item && pin.column == left_column && *source == right_item {
            pin.peer_column = Some(right_column.clone());
        } else if *item == right_item && pin.column == right_column && *source == left_item {
            pin.peer_column = Some(left_column.clone());
        }
    }
}
