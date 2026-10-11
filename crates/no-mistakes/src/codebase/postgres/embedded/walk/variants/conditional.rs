use super::super::ScopeVisitor;
use super::conditions::{choice_id, truth};
use super::{alternatives, bounded, expression, Recovered};
use oxc_ast::ast::{ConditionalExpression, Expression};

pub(super) fn recover(
    visitor: &ScopeVisitor<'_>,
    branch: &ConditionalExpression<'_>,
    depth: u8,
    constraints: &[(u64, u32)],
) -> Option<Vec<Recovered>> {
    if visitor.expression_mutates_builder(&branch.test) {
        return None;
    }
    let (id, truth_arm) = choice_id(visitor, &branch.test, branch.span.start);
    let constrained = constraints
        .iter()
        .find(|(other, _)| *other == id)
        .map(|(_, choice)| *choice == truth_arm);
    if let Some(taken) = constrained.or_else(|| truth(&branch.test)) {
        let arm = if taken {
            &branch.consequent
        } else {
            &branch.alternate
        };
        return expression::recover(visitor, arm, depth - 1, constraints).map(mark);
    }
    if let Some(tests) = expression::recover(visitor, &branch.test, depth - 1, constraints) {
        let (yes, no): (Vec<_>, Vec<_>) = tests.into_iter().partition(Recovered::truth);
        let left = if yes.is_empty() {
            Vec::new()
        } else {
            recover_arm(
                visitor,
                &branch.consequent,
                depth - 1,
                yes,
                (id, truth_arm),
                constraints,
            )?
        };
        let right = if no.is_empty() {
            Vec::new()
        } else {
            recover_arm(
                visitor,
                &branch.alternate,
                depth - 1,
                no,
                (id, 1 - truth_arm),
                constraints,
            )?
        };
        return alternatives(left, right).map(mark);
    }
    let mut yes = constraints.to_vec();
    yes.push((id, truth_arm));
    let mut no = constraints.to_vec();
    no.push((id, 1 - truth_arm));
    let mut left = expression::recover(visitor, &branch.consequent, depth - 1, &yes)?;
    let mut right = expression::recover(visitor, &branch.alternate, depth - 1, &no)?;
    for value in &mut left {
        value.choices.push((id, truth_arm));
    }
    for value in &mut right {
        value.choices.push((id, 1 - truth_arm));
    }
    alternatives(left, right)
}
fn mark(mut values: Vec<Recovered>) -> Vec<Recovered> {
    for value in &mut values {
        value.enumerated = true;
    }
    values
}
fn recover_arm(
    visitor: &ScopeVisitor<'_>,
    arm: &Expression<'_>,
    depth: u8,
    tests: Vec<Recovered>,
    choice: (u64, u32),
    constraints: &[(u64, u32)],
) -> Option<Vec<Recovered>> {
    let mut values = Vec::new();
    for test in tests {
        let mut path = constraints.to_vec();
        path.extend(test.choices);
        path.push(choice);
        let recovered = expression::recover(visitor, arm, depth, &path)?;
        values.extend(recovered.into_iter().map(|mut value| {
            value.choices.extend(path.iter().copied());
            value
        }));
    }
    bounded(values)
}
