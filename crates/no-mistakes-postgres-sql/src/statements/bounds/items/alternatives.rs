//! Preserve disjunctive key proofs without pooling columns from different branches.
use super::super::{pins, Scope};
use crate::idents::unwrap_expr;
use crate::statements::{SqlBoundItem, SqlBoundItemKind};
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
    let count = branches.len();
    for (index, item) in items.iter_mut().enumerate() {
        item.pin_alternatives = count;
        if resolver.is_table(index) {
            item.alternative_predicates = vec![Vec::new(); count];
        }
    }
    for (alternative, branch) in branches.into_iter().enumerate() {
        let mut found = Vec::new();
        pins::extract_all(&branch, resolver, &restricted, scope, positions, &mut found);
        for (index, mut pin) in found {
            if !matches!(
                &items[index].kind,
                SqlBoundItemKind::Table(_) | SqlBoundItemKind::Query(_)
            ) {
                continue;
            }
            pin.alternative = Some(alternative);
            if !items[index].pins.contains(&pin) {
                items[index].pins.push(pin);
            }
        }
        for expr in &branch {
            for (index, item) in items.iter_mut().enumerate() {
                if !resolver.is_table(index) {
                    continue;
                }
                for conjunct in pins::item_predicates(resolver, expr, index) {
                    let slot = &mut item.alternative_predicates[alternative];
                    if !slot.contains(&conjunct) {
                        slot.push(conjunct);
                    }
                }
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
