mod calls;
mod writes;

pub(super) use writes::{contains_identity_writes, independent_argument};

use super::super::super::ScopeVisitor;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashMap;
use oxc_ast::ast::{AssignmentExpression, AssignmentOperator, AssignmentTarget, Expression};

pub(super) type Targets = Vec<(String, Option<u32>)>;

/// Only the final interpolation uses this transient projection. It follows
/// assignments in evaluation order without changing the extractor's bindings.
#[derive(Clone, Default)]
pub(super) struct EffectIdentities {
    assigned: FxHashMap<String, Option<Targets>>,
}

impl EffectIdentities {
    pub(super) fn join(&mut self, visitor: &ScopeVisitor<'_>, before: &Self, other: Self) {
        for name in self
            .assigned
            .keys()
            .chain(other.assigned.keys())
            .cloned()
            .collect::<Vec<_>>()
        {
            let initial = before.binding(visitor, &name);
            let left = self
                .assigned
                .get(&name)
                .cloned()
                .unwrap_or_else(|| initial.clone());
            let right = other.assigned.get(&name).cloned().unwrap_or(initial);
            let value = match (left, right) {
                (Some(mut left), Some(right)) => {
                    for value in right {
                        if !left.contains(&value) {
                            left.push(value);
                        }
                    }
                    Some(left)
                }
                _ => None,
            };
            self.assigned.insert(name, value);
        }
    }

    fn binding(&self, visitor: &ScopeVisitor<'_>, name: &str) -> Option<Targets> {
        if let Some(value) = self.assigned.get(name) {
            return value.clone();
        }
        let targets = visitor
            .lookup(name)
            .filter(|binding| {
                binding.builder_identity.is_some()
                    || binding
                        .variants
                        .as_ref()
                        .is_some_and(|values| values.iter().any(|value| value.fragment))
            })
            .map(|binding| (name.to_owned(), binding.builder_identity));
        Some(targets.into_iter().collect())
    }

    pub(super) fn assign(
        &mut self,
        visitor: &ScopeVisitor<'_>,
        assignment: &AssignmentExpression<'_>,
    ) -> bool {
        let Some(name) = assigned_name(&assignment.left) else {
            return true;
        };
        let targets = if assignment.operator == AssignmentOperator::Assign {
            self.value(visitor, &assignment.right)
        } else {
            None
        };
        self.assigned.insert(name.to_owned(), targets);
        false
    }

    pub(super) fn target(&self, name: String, identity: Option<u32>) -> Option<Targets> {
        self.assigned
            .get(&name)
            .cloned()
            .unwrap_or_else(|| Some(vec![(name, identity)]))
    }

    pub(super) fn value(
        &self,
        visitor: &ScopeVisitor<'_>,
        expression: &Expression<'_>,
    ) -> Option<Targets> {
        let expression = unwrap_ts_wrappers(expression);
        match expression {
            Expression::Identifier(id) => self.binding(visitor, id.name.as_str()),
            Expression::AssignmentExpression(assignment)
                if assignment.operator == AssignmentOperator::Assign =>
            {
                self.value(visitor, &assignment.right)
            }
            Expression::ConditionalExpression(branch)
                if !writes::contains_identity_writes(&branch.test) =>
            {
                self.union(visitor, &branch.consequent, &branch.alternate)
            }
            Expression::LogicalExpression(branch)
                if !writes::contains_identity_writes(&branch.left) =>
            {
                self.union(visitor, &branch.left, &branch.right)
            }
            Expression::SequenceExpression(sequence) => {
                let (last, earlier) = sequence
                    .expressions
                    .split_last()
                    .expect("sequence expressions have a value");
                if earlier.iter().any(writes::contains_identity_writes)
                    && !writes::copied_scalar(last)
                {
                    return None;
                }
                self.value(visitor, last)
            }
            Expression::TaggedTemplateExpression(_) => visitor
                .builder_identity_for_init(expression)
                .map(|identity| vec![(String::new(), Some(identity))]),
            Expression::CallExpression(call) => self.call_value(visitor, call, expression),
            Expression::StringLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::BigIntLiteral(_) => Some(Vec::new()),
            _ => None,
        }
    }

    fn union(
        &self,
        visitor: &ScopeVisitor<'_>,
        left: &Expression<'_>,
        right: &Expression<'_>,
    ) -> Option<Targets> {
        let mut targets = self.value(visitor, left)?;
        for target in self.value(visitor, right)? {
            if !targets.contains(&target) {
                targets.push(target);
            }
        }
        Some(targets)
    }
}

fn assigned_name<'a>(target: &'a AssignmentTarget<'_>) -> Option<&'a str> {
    let expression = match target {
        AssignmentTarget::AssignmentTargetIdentifier(id) => return Some(id.name.as_str()),
        AssignmentTarget::TSAsExpression(value) => &value.expression,
        AssignmentTarget::TSSatisfiesExpression(value) => &value.expression,
        AssignmentTarget::TSNonNullExpression(value) => &value.expression,
        AssignmentTarget::TSTypeAssertion(value) => &value.expression,
        _ => return None,
    };
    match unwrap_ts_wrappers(expression) {
        Expression::Identifier(id) => Some(id.name.as_str()),
        _ => None,
    }
}
