mod refs;
mod sets;

#[cfg(test)]
mod tests;

use crate::codebase::postgres::idents::unwrap_expr;
use sqlparser::ast::{BinaryOperator, Expr};
use std::collections::BTreeSet;

pub(super) struct Instance<'a> {
    pub table: String,
    pub alias: Option<String>,
    pub joined: bool,
    pub line: usize,
    pub extra_on: Vec<&'a Expr>,
    pub on: Option<&'a Expr>,
}

#[derive(Default)]
struct Cols {
    proven: BTreeSet<String>,
    pending: BTreeSet<String>,
}

struct Ctx<'a, P> {
    instances: &'a [Instance<'a>],
    index: usize,
    from_items: usize,
    all_base: bool,
    positions: Option<&'a [(u32, u32)]>,
    projection: &'a P,
}

pub(super) fn constrain_projected(
    instances: &[Instance<'_>],
    selection: Option<&Expr>,
    from_items: usize,
    all_base: bool,
    positions: super::super::value::PlaceholderPositions<'_>,
    projection: &impl super::Projection,
) -> Vec<(Vec<String>, Vec<String>)> {
    instances
        .iter()
        .enumerate()
        .map(|(index, instance)| {
            let ctx = Ctx {
                instances,
                index,
                from_items,
                all_base,
                positions,
                projection,
            };
            let mut cols = Cols::default();
            if instance.joined {
                if let Some(on) = instance.on {
                    cols = sets::union(cols, columns_from_expr(on, &ctx));
                }
            }
            for on in &instance.extra_on {
                cols = sets::union(cols, columns_from_expr(on, &ctx));
            }
            if let Some(selection) = selection {
                cols = sets::union(cols, columns_from_expr(selection, &ctx));
            }
            (
                cols.proven.into_iter().collect(),
                cols.pending.into_iter().collect(),
            )
        })
        .collect()
}

fn columns_from_expr(predicate: &Expr, ctx: &Ctx<'_, impl super::Projection>) -> Cols {
    match unwrap_expr(predicate) {
        Expr::BinaryOp { left, op, right } => match op {
            BinaryOperator::And => {
                sets::union(columns_from_expr(left, ctx), columns_from_expr(right, ctx))
            }
            BinaryOperator::Or => {
                sets::intersect(columns_from_expr(left, ctx), columns_from_expr(right, ctx))
            }
            BinaryOperator::Eq
            | BinaryOperator::Gt
            | BinaryOperator::Lt
            | BinaryOperator::GtEq
            | BinaryOperator::LtEq => compare(predicate, left, right, ctx),
            _ => Cols::default(),
        },
        Expr::Between {
            expr,
            negated: false,
            low,
            high,
        } if !refs::references(low, ctx) && !refs::references(high, ctx) => {
            from_hit(predicate, None, column_hit(expr, ctx), ctx)
        }
        Expr::InList {
            expr,
            negated: false,
            list,
        } if !list.iter().any(|item| refs::references(item, ctx)) => {
            from_hit(predicate, None, column_hit(expr, ctx), ctx)
        }
        Expr::InSubquery {
            expr,
            negated: false,
            ..
        } => from_hit(predicate, None, column_hit(expr, ctx), ctx),
        Expr::AnyOp {
            left,
            compare_op: BinaryOperator::Eq,
            right,
            ..
        } if !refs::references(right, ctx) => {
            from_hit(predicate, Some(right), column_hit(left, ctx), ctx)
        }
        _ => Cols::default(),
    }
}

fn compare(
    predicate: &Expr,
    left: &Expr,
    right: &Expr,
    ctx: &Ctx<'_, impl super::Projection>,
) -> Cols {
    let mut cols = Cols::default();
    absorb(&mut cols, predicate, left, right, ctx);
    absorb(&mut cols, predicate, right, left, ctx);
    cols
}

mod hits;
use hits::{absorb, column_hit, from_hit};
