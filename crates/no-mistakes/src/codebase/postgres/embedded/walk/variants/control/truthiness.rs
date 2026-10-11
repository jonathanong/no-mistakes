use super::super::super::ScopeVisitor;
use super::super::{Recovered, ValueKind};
use super::branch_paths;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, LogicalExpression, LogicalOperator, UnaryOperator};

type Paths = Vec<Vec<(u64, u32)>>;

pub(super) fn branches(
    visitor: &ScopeVisitor<'_>,
    test: &Expression<'_>,
    paths: &[Vec<(u64, u32)>],
    id: u64,
    truth_arm: u32,
) -> (Paths, Paths) {
    let mut expression = unwrap_ts_wrappers(test);
    let mut invert = false;
    while let Expression::UnaryExpression(unary) = expression {
        if unary.operator != UnaryOperator::LogicalNot {
            break;
        }
        invert = !invert;
        expression = unwrap_ts_wrappers(&unary.argument);
    }
    recovered_paths(visitor, expression, paths, id, truth_arm, |value| {
        value.truth() != invert
    })
}

pub(super) fn logical_branches(
    visitor: &ScopeVisitor<'_>,
    expression: &LogicalExpression<'_>,
    paths: &[Vec<(u64, u32)>],
    id: u64,
    right_arm: u32,
) -> (Paths, Paths) {
    match expression.operator {
        LogicalOperator::Coalesce => {
            recovered_paths(visitor, &expression.left, paths, id, right_arm, |value| {
                value.value == ValueKind::Null
            })
        }
        LogicalOperator::And => branches(visitor, &expression.left, paths, id, right_arm),
        LogicalOperator::Or => {
            let (retained, replaced) =
                branches(visitor, &expression.left, paths, id, 1 - right_arm);
            (replaced, retained)
        }
    }
}

fn recovered_paths(
    visitor: &ScopeVisitor<'_>,
    test: &Expression<'_>,
    paths: &[Vec<(u64, u32)>],
    id: u64,
    truth_arm: u32,
    truth: impl Fn(&Recovered) -> bool,
) -> (Paths, Paths) {
    let Some(values) = visitor.recover_variants(test) else {
        return (
            branch_paths(paths, id, truth_arm),
            branch_paths(paths, id, 1 - truth_arm),
        );
    };
    let mut yes = Vec::new();
    let mut no = Vec::new();
    for value in values {
        let taken = truth(&value);
        let mut compatible = paths.to_vec();
        // A branch-assigned guard can lose its condition key while retaining
        // recovered choices shared with SQL assigned on those same paths.
        for (choice_id, choice_arm) in value.choices {
            compatible = branch_paths(&compatible, choice_id, choice_arm);
        }
        let arm = if taken { truth_arm } else { 1 - truth_arm };
        let output = if taken { &mut yes } else { &mut no };
        for path in branch_paths(&compatible, id, arm) {
            if !output.contains(&path) {
                output.push(path);
            }
        }
    }
    (yes, no)
}
