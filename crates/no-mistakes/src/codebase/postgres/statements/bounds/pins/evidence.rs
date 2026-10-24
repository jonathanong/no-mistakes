use super::Resolver;
use sqlparser::ast::{BinaryOperator, Expr};

pub(in super::super) fn collect(
    expr: &Expr,
    resolver: &Resolver,
    restricted: &[usize],
    conjuncts: &mut Vec<(usize, Expr)>,
    nonnull: &mut Vec<(usize, String)>,
) {
    let expr = crate::codebase::postgres::idents::unwrap_expr(expr);
    if let Expr::BinaryOp {
        left,
        op: BinaryOperator::And,
        right,
    } = expr
    {
        collect(left, resolver, restricted, conjuncts, nonnull);
        collect(right, resolver, restricted, conjuncts, nonnull);
        return;
    }
    for &item in restricted {
        if resolver.is_table(item) {
            if let Some(canonical) = resolver.canonical_for_item(expr, item) {
                conjuncts.push((item, canonical));
            }
        }
    }
    if let Expr::BinaryOp {
        left,
        op: BinaryOperator::Eq,
        right,
    } = expr
    {
        for side in [left.as_ref(), right.as_ref()] {
            if let Some((item, column)) = resolver.column(side) {
                if restricted.contains(&item) && resolver.is_table(item) {
                    nonnull.push((item, column));
                }
            }
        }
    }
}
