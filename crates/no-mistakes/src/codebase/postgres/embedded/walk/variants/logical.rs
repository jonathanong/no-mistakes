use super::super::ScopeVisitor;
use super::conditions::{choice_id, truth};
use super::{alternatives, bounded, expression, Recovered, ValueKind};
use oxc_ast::ast::{Expression, LogicalExpression, LogicalOperator};

pub(super) fn recover(
    visitor: &ScopeVisitor<'_>,
    branch: &LogicalExpression<'_>,
    depth: u8,
    line: u32,
    constraints: &[(u64, u32)],
) -> Option<Vec<Recovered>> {
    if visitor.expression_mutates_builder(&branch.left) {
        return None;
    }
    let left = expression::recover(visitor, &branch.left, depth - 1, constraints);
    if branch.operator == LogicalOperator::And {
        if let Some(left) = left {
            let (truthy, falsy): (Vec<_>, Vec<_>) = left.into_iter().partition(Recovered::truth);
            let falsy = falsy.into_iter().map(absent).collect();
            if truthy.is_empty() {
                return Some(falsy);
            }
            let right = replace(visitor, truthy, &branch.right, depth - 1, constraints)?;
            if !right.iter().all(|value| value.fragment) {
                return None;
            }
            return alternatives(falsy, right);
        }
        let mut omitted = Recovered::empty(line);
        omitted.fragment = true;
        omitted.value = ValueKind::Absent;
        let (id, truth_arm) = choice_id(visitor, &branch.left, branch.span.start);
        let taken = constraints
            .iter()
            .find(|(other, _)| *other == id)
            .map(|(_, choice)| *choice == truth_arm)
            .or_else(|| truth(&branch.left));
        if taken == Some(false) {
            return Some(vec![omitted]);
        }
        let mut path = constraints.to_vec();
        path.push((id, truth_arm));
        let mut right = expression::recover(visitor, &branch.right, depth - 1, &path)?;
        if !right.iter().all(|value| value.fragment) {
            return None;
        }
        if taken == Some(true) {
            return Some(right);
        }
        for value in &mut right {
            value.choices.push((id, truth_arm));
        }
        omitted.choices.push((id, 1 - truth_arm));
        return alternatives(vec![omitted], right);
    }
    if let Some(left) = left {
        let (retained, replaced): (Vec<_>, Vec<_>) = left.into_iter().partition(|value| {
            if branch.operator == LogicalOperator::Coalesce {
                value.value != ValueKind::Null
            } else {
                value.truth()
            }
        });
        if replaced.is_empty() {
            return Some(retained);
        }
        return alternatives(
            retained,
            replace(visitor, replaced, &branch.right, depth - 1, constraints)?,
        );
    }
    if branch.operator == LogicalOperator::Or && truth(&branch.left) == Some(false) {
        expression::recover(visitor, &branch.right, depth - 1, constraints)
    } else {
        None
    }
}
fn replace(
    visitor: &ScopeVisitor<'_>,
    left: Vec<Recovered>,
    right: &Expression<'_>,
    depth: u8,
    constraints: &[(u64, u32)],
) -> Option<Vec<Recovered>> {
    let mut result = Vec::new();
    for value in left {
        let mut path = constraints.to_vec();
        path.extend(value.choices);
        let recovered = expression::recover(visitor, right, depth, &path)?;
        result.extend(recovered.into_iter().map(|mut value| {
            value.choices.extend(path.iter().copied());
            value
        }));
    }
    bounded(result)
}
fn absent(mut value: Recovered) -> Recovered {
    value.sql.clear();
    value.origins.clear();
    value.positions.clear();
    value.fragment = true;
    value.value = ValueKind::Absent;
    value
}
