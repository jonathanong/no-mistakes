use super::super::ScopeVisitor;
use super::{alternatives, Recovered};
use crate::fx::FxHashMap;
use oxc_ast::ast::{ConditionalExpression, IfStatement, LogicalExpression};
use oxc_ast_visit::Visit;
mod paths;
mod truthiness;
pub(in crate::codebase::postgres::embedded::walk) use paths::branch_paths;

pub(super) type Snapshot = Vec<FxHashMap<String, Option<Vec<Recovered>>>>;
impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn variant_snapshot(&self) -> Snapshot {
        self.scopes
            .iter()
            .map(|scope| {
                scope
                    .iter()
                    .map(|(name, state)| (name.clone(), state.variants.clone()))
                    .collect()
            })
            .collect()
    }
    pub(in crate::codebase::postgres::embedded::walk) fn restore_variants(
        &mut self,
        snapshot: &Snapshot,
    ) {
        for (scope, old) in self.scopes.iter_mut().zip(snapshot) {
            for (name, state) in scope {
                state.variants = old.get(name).cloned().flatten();
            }
        }
    }
    pub(in crate::codebase::postgres::embedded::walk) fn join_variants(
        &mut self,
        left: &Snapshot,
        right: &Snapshot,
        id: u64,
        arms: (u32, u32),
    ) {
        for (index, scope) in self.scopes.iter_mut().enumerate() {
            for (name, state) in scope {
                let a = left
                    .get(index)
                    .and_then(|scope| scope.get(name))
                    .cloned()
                    .flatten();
                let b = right
                    .get(index)
                    .and_then(|scope| scope.get(name))
                    .cloned()
                    .flatten();
                state.variants = if a == b {
                    a
                } else {
                    a.zip(b).and_then(|(mut a, mut b)| {
                        for value in &mut a {
                            if !value.choices.iter().any(|(other, _)| *other == id) {
                                value.choices.push((id, arms.0));
                            }
                        }
                        for value in &mut b {
                            if !value.choices.iter().any(|(other, _)| *other == id) {
                                value.choices.push((id, arms.1));
                            }
                        }
                        alternatives(a, b)
                    })
                };
            }
        }
    }
}

pub(in crate::codebase::postgres::embedded::walk) fn if_statement<'a>(
    visitor: &mut ScopeVisitor<'a>,
    statement: &IfStatement<'a>,
) {
    let (id, truth_arm) =
        super::conditions::choice_id(visitor, &statement.test, statement.span.start);
    let taken = super::conditions::truth_at(visitor, &statement.test);
    visitor.visit_expression(&statement.test);
    let original = visitor.variant_snapshot();
    let paths = visitor.variant_paths.clone();
    let (yes, no) = truthiness::branches(visitor, &statement.test, &paths, id, truth_arm);
    let taken = paths::taken(&yes, &no).or(taken);
    visitor.restore_variants(&paths::restrict(&original, &yes));
    visitor.variant_paths = yes;
    visitor.with_control_flow(|visitor| visitor.visit_statement(&statement.consequent));
    let left = visitor.variant_snapshot();
    visitor.restore_variants(&paths::restrict(&original, &no));
    visitor.variant_paths = no;
    if let Some(alternate) = &statement.alternate {
        visitor.with_control_flow(|visitor| visitor.visit_statement(alternate));
    }
    let right = visitor.variant_snapshot();
    visitor.variant_paths = paths;
    match taken {
        Some(true) => select(visitor, &left, &right),
        Some(false) => select(visitor, &right, &left),
        None => visitor.join_variants(&left, &right, id, (truth_arm, 1 - truth_arm)),
    }
}

pub(in crate::codebase::postgres::embedded::walk) fn conditional<'a>(
    visitor: &mut ScopeVisitor<'a>,
    expression: &ConditionalExpression<'a>,
) {
    let (id, truth_arm) =
        super::conditions::choice_id(visitor, &expression.test, expression.span.start);
    let taken = super::conditions::truth_at(visitor, &expression.test);
    visitor.visit_expression(&expression.test);
    let original = visitor.variant_snapshot();
    let paths = visitor.variant_paths.clone();
    let (yes, no) = truthiness::branches(visitor, &expression.test, &paths, id, truth_arm);
    let taken = paths::taken(&yes, &no).or(taken);
    visitor.restore_variants(&paths::restrict(&original, &yes));
    visitor.variant_paths = yes;
    visitor.with_control_flow(|visitor| visitor.visit_expression(&expression.consequent));
    let left = visitor.variant_snapshot();
    visitor.restore_variants(&paths::restrict(&original, &no));
    visitor.variant_paths = no;
    visitor.with_control_flow(|visitor| visitor.visit_expression(&expression.alternate));
    let right = visitor.variant_snapshot();
    visitor.variant_paths = paths;
    match taken {
        Some(true) => select(visitor, &left, &right),
        Some(false) => select(visitor, &right, &left),
        None => visitor.join_variants(&left, &right, id, (truth_arm, 1 - truth_arm)),
    }
}

pub(in crate::codebase::postgres::embedded::walk) fn logical<'a>(
    visitor: &mut ScopeVisitor<'a>,
    expression: &LogicalExpression<'a>,
) {
    let (id, truth_arm) = if expression.operator == oxc_ast::ast::LogicalOperator::Coalesce {
        (u64::from(expression.span.start), 0)
    } else {
        super::conditions::choice_id(visitor, &expression.left, expression.span.start)
    };
    let right_arm = if expression.operator == oxc_ast::ast::LogicalOperator::And {
        truth_arm
    } else {
        1 - truth_arm
    };
    let take_right = match expression.operator {
        oxc_ast::ast::LogicalOperator::And => {
            super::conditions::truth_at(visitor, &expression.left)
        }
        oxc_ast::ast::LogicalOperator::Or => {
            super::conditions::truth_at(visitor, &expression.left).map(|truth| !truth)
        }
        oxc_ast::ast::LogicalOperator::Coalesce => {
            super::conditions::nullish_at(visitor, &expression.left)
        }
    };
    visitor.visit_expression(&expression.left);
    let original = visitor.variant_snapshot();
    let paths = visitor.variant_paths.clone();
    let (right_paths, omitted_paths) =
        truthiness::logical_branches(visitor, expression, &paths, id, right_arm);
    let take_right = paths::taken(&right_paths, &omitted_paths).or(take_right);
    visitor.restore_variants(&paths::restrict(&original, &right_paths));
    visitor.variant_paths = right_paths;
    visitor.with_control_flow(|visitor| visitor.visit_expression(&expression.right));
    let right = visitor.variant_snapshot();
    let original = paths::restrict(&original, &omitted_paths);
    visitor.variant_paths = paths;

    match take_right {
        Some(true) => select(visitor, &right, &original),
        Some(false) => select(visitor, &original, &right),
        None => visitor.join_variants(&original, &right, id, (1 - right_arm, right_arm)),
    }
}

fn select(visitor: &mut ScopeVisitor<'_>, selected: &Snapshot, other: &Snapshot) {
    visitor.restore_variants(selected);
    for (index, scope) in visitor.scopes.iter_mut().enumerate() {
        for (name, state) in scope {
            let selected = selected.get(index).and_then(|scope| scope.get(name));
            let other = other.get(index).and_then(|scope| scope.get(name));
            if selected != other {
                if let Some(values) = &mut state.variants {
                    for value in values {
                        value.enumerated = true;
                    }
                }
            }
        }
    }
}
