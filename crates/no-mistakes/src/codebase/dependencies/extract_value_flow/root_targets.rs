use super::*;
use oxc_ast::ast::{AssignmentTargetMaybeDefault, AssignmentTargetProperty};

pub(super) fn for_each_root(target: &AssignmentTarget<'_>, mut visit: impl FnMut(&str, u32)) {
    walk_roots(target, &mut visit);
}

fn walk_roots<F: FnMut(&str, u32)>(target: &AssignmentTarget<'_>, visit: &mut F) {
    match target {
        AssignmentTarget::AssignmentTargetIdentifier(identifier) => {
            visit(identifier.name.as_str(), target.span().start);
        }
        AssignmentTarget::ArrayAssignmentTarget(array) => {
            for element in array.elements.iter().flatten() {
                maybe_default(element, visit);
            }
            if let Some(rest) = &array.rest {
                walk_roots(&rest.target, visit);
            }
        }
        AssignmentTarget::ObjectAssignmentTarget(object) => {
            for property in &object.properties {
                match property {
                    AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(property) => {
                        visit(property.binding.name.as_str(), property.binding.span.start);
                    }
                    AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
                        maybe_default(&property.binding, visit);
                    }
                }
            }
            if let Some(rest) = &object.rest {
                walk_roots(&rest.target, visit);
            }
        }
        AssignmentTarget::TSAsExpression(value) => wrapped(&value.expression, target, visit),
        AssignmentTarget::TSSatisfiesExpression(value) => wrapped(&value.expression, target, visit),
        AssignmentTarget::TSNonNullExpression(value) => wrapped(&value.expression, target, visit),
        AssignmentTarget::TSTypeAssertion(value) => wrapped(&value.expression, target, visit),
        // Every member target has a nonempty path. Its expression is still
        // visited normally; root metadata does not evaluate that expression.
        _ => {}
    }
}

fn maybe_default<F: FnMut(&str, u32)>(target: &AssignmentTargetMaybeDefault<'_>, visit: &mut F) {
    if let AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(default) = target {
        walk_roots(&default.binding, visit);
    } else {
        walk_roots(target.to_assignment_target(), visit);
    }
}

fn wrapped<F: FnMut(&str, u32)>(
    expression: &Expression<'_>,
    target: &AssignmentTarget<'_>,
    visit: &mut F,
) {
    if let Expression::Identifier(identifier) =
        crate::codebase::ts_source::unwrap_ts_wrappers(expression)
    {
        visit(identifier.name.as_str(), target.span().start);
    }
}

#[cfg(test)]
#[path = "root_targets_tests.rs"]
mod tests;
