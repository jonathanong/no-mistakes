//! Preserve disjunctive key proofs without pooling columns from different branches.
use super::super::{pins, Scope};
use crate::codebase::postgres::idents::unwrap_expr;
use crate::codebase::postgres::statements::SqlBoundItem;
use sqlparser::ast::{BinaryOperator, Expr};

// DNF may grow exponentially. Larger expressions retain only ordinary conjunctive pins.
const MAX_ALTERNATIVES: usize = 32;

pub(super) fn collect(
    expr: &Expr,
    resolver: &pins::Resolver,
    scope: &Scope,
    positions: super::super::super::value::PlaceholderPositions<'_>,
    items: &mut [SqlBoundItem],
) {
    if !contains_or(expr) {
        return;
    }
    let Some(branches) = branches(expr).filter(|branches| branches.len() > 1) else {
        return;
    };
    let restricted: Vec<_> = (0..items.len()).collect();
    for item in items.iter_mut() {
        item.pin_alternatives = branches.len();
    }
    for (alternative, branch) in branches.into_iter().enumerate() {
        let mut found = Vec::new();
        for conjunct in branch {
            pins::extract(
                conjunct,
                resolver,
                &restricted,
                scope,
                positions,
                &mut found,
            );
        }
        for (index, mut pin) in found {
            pin.alternative = Some(alternative);
            if !items[index].pins.contains(&pin) {
                items[index].pins.push(pin);
            }
        }
    }
}

fn branches(expr: &Expr) -> Option<Vec<Vec<&Expr>>> {
    match unwrap_expr(expr) {
        Expr::BinaryOp { left, op, right }
            if matches!(op, BinaryOperator::And | BinaryOperator::Or) =>
        {
            let left = branches(left)?;
            let right = branches(right)?;
            if *op == BinaryOperator::Or {
                (left.len() + right.len() <= MAX_ALTERNATIVES)
                    .then(|| left.into_iter().chain(right).collect())
            } else if left.len() * right.len() <= MAX_ALTERNATIVES {
                Some(
                    left.iter()
                        .flat_map(|left| {
                            right
                                .iter()
                                .map(move |right| left.iter().chain(right).copied().collect())
                        })
                        .collect(),
                )
            } else {
                None
            }
        }
        _ => Some(vec![vec![expr]]),
    }
}

fn contains_or(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::BinaryOp {
            op: BinaryOperator::Or,
            ..
        } => true,
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => contains_or(left) || contains_or(right),
        _ => false,
    }
}
