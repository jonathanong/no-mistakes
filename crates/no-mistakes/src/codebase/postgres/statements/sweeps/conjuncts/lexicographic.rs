use super::{bound, column, flatten, Cursor};
use crate::codebase::postgres::idents::unwrap_expr;
use sqlparser::ast::{BinaryOperator, Expr};

/// Match the expanded tuple comparison, retaining bind identity in every prefix.
pub(super) fn cursor(
    expr: &Expr,
    names: &[String],
    order_columns: &[String],
    order_ascending: &[Option<bool>],
    int4_bindings: &super::super::Int4Bindings,
    recovered_placeholder_positions: &[(u32, u32)],
) -> Option<Cursor> {
    fn arms<'a>(expr: &'a Expr, out: &mut Vec<&'a Expr>) {
        match unwrap_expr(expr) {
            Expr::BinaryOp {
                left,
                op: BinaryOperator::Or,
                right,
            } => {
                arms(left, out);
                arms(right, out);
            }
            other => out.push(other),
        }
    }
    let mut alternatives = Vec::new();
    arms(expr, &mut alternatives);
    let mut alternatives: Vec<_> = alternatives
        .into_iter()
        .map(|arm| {
            let mut terms = Vec::new();
            flatten(arm, &mut terms);
            terms
        })
        .collect();
    // The number of equality-prefix terms identifies an arm's position in the
    // expanded cursor. OR order is semantically irrelevant, so normalize it
    // before checking the complete prefix chain.
    alternatives.sort_by_key(Vec::len);
    let last_arm = alternatives.len().checked_sub(1)?;
    let mut keys: Vec<(String, super::super::bind_identity::Identity<'_>)> = Vec::new();
    let mut direction = None;
    let mut first_lower = None;
    for (index, terms) in alternatives.into_iter().enumerate() {
        if terms.len() != index + 1 {
            return None;
        }
        for (prefix, term) in terms.iter().take(index).enumerate() {
            let Expr::BinaryOp {
                left,
                op: BinaryOperator::Eq,
                right,
            } = unwrap_expr(term)
            else {
                return None;
            };
            let (column, identity, _) = operands(
                left,
                right,
                names,
                int4_bindings,
                recovered_placeholder_positions,
            )?;
            if column != keys[prefix].0 || identity != keys[prefix].1 {
                return None;
            }
        }
        let Expr::BinaryOp { left, op, right } = unwrap_expr(terms[index]) else {
            return None;
        };
        let (column, identity, reversed) = operands(
            left,
            right,
            names,
            int4_bindings,
            recovered_placeholder_positions,
        )?;
        let lower = match op {
            BinaryOperator::Gt => !reversed,
            BinaryOperator::Lt => reversed,
            BinaryOperator::GtEq if index == last_arm => !reversed,
            BinaryOperator::LtEq if index == last_arm => reversed,
            _ => return None,
        };
        let ascending = order_ascending.get(index).copied().flatten()?;
        // Every arm must move the same way through its own ordered key. The public
        // lower/upper bound still follows the first key, so opposite cursors form a window.
        let forward = lower == ascending;
        if direction.is_some_and(|direction| direction != forward) {
            return None;
        }
        direction = Some(forward);
        first_lower.get_or_insert(lower);
        keys.push((column, identity));
    }
    // An expanded comparison is contiguous only in its leading ORDER BY key sequence.
    if !keys
        .iter()
        .map(|key| &key.0)
        .eq(order_columns.iter().take(keys.len()))
    {
        return None;
    }
    Some(Cursor {
        columns: keys.into_iter().map(|key| key.0).collect(),
        bound: bound(first_lower?),
        optional: false,
    })
}

/// Determine the column and normalized bind once for either comparison orientation.
fn operands<'a>(
    left: &'a Expr,
    right: &'a Expr,
    names: &[String],
    bindings: &super::super::Int4Bindings,
    positions: &[(u32, u32)],
) -> Option<(String, super::super::bind_identity::Identity<'a>, bool)> {
    let forward = column(left, names, positions)
        .zip(super::super::bind_identity::of(right, bindings, positions));
    let reversed = column(right, names, positions)
        .zip(super::super::bind_identity::of(left, bindings, positions));
    match (forward, reversed) {
        (Some((column, identity)), None) => Some((column, identity, false)),
        (None, Some((column, identity))) => Some((column, identity, true)),
        _ => None,
    }
}
