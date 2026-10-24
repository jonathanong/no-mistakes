use super::resolver::{Resolver, Sourced};
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
        pin(left, resolver.source(right, item), null_safe);
    }
    if let Some((item, _)) = resolver.column(right) {
        pin(right, resolver.source(left, item), null_safe);
    }
}
