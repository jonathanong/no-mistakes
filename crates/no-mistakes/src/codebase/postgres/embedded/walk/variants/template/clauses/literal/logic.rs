//! `&&`, `||`, and `??` between literal SQL clauses.
//! A reached unknown branch abstains. A branch JavaScript will not evaluate
//! does not.

use super::super::super::super::super::ScopeVisitor;
use super::super::super::super::limit::bounded;
use super::super::super::super::Recovered;
use super::clause;
use oxc_ast::ast::{Expression, LogicalExpression, LogicalOperator};

pub(super) fn recover(
    visitor: &ScopeVisitor<'_>,
    branch: &LogicalExpression<'_>,
    depth: u8,
    constraints: &[(u64, u32)],
) -> Option<Vec<Recovered>> {
    if visitor.expression_mutates_builder(&branch.left) {
        return None;
    }
    let left = clause(visitor, &branch.left, depth - 1, constraints)?;
    match branch.operator {
        // A recovered clause is a string, so it is never nullish. "" stays.
        LogicalOperator::Coalesce => Some(left),
        LogicalOperator::And => apply(visitor, &branch.right, depth, constraints, left, false),
        LogicalOperator::Or => apply(visitor, &branch.right, depth, constraints, left, true),
    }
}

/// `keep_truthy` is `||`. `&&` keeps only the falsy strings.
fn apply(
    visitor: &ScopeVisitor<'_>,
    right: &Expression<'_>,
    depth: u8,
    constraints: &[(u64, u32)],
    left: Vec<Recovered>,
    keep_truthy: bool,
) -> Option<Vec<Recovered>> {
    let (kept, replaced): (Vec<_>, Vec<_>) = left.into_iter().partition(|value| {
        let truthy = !value.sql.is_empty();
        if keep_truthy {
            truthy
        } else {
            !truthy
        }
    });
    if replaced.is_empty() {
        return bounded(kept);
    }
    let mut out = kept;
    for value in replaced {
        let mut path = constraints.to_vec();
        path.extend(value.choices.iter().copied());
        let mut arm = clause(visitor, right, depth - 1, &path)?;
        for item in &mut arm {
            for choice in &value.choices {
                if !item.choices.contains(choice) {
                    item.choices.push(*choice);
                }
            }
        }
        out.extend(arm);
    }
    bounded(out)
}
