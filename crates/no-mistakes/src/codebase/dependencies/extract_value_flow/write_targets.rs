use super::*;
use oxc_ast::ast::{AssignmentTargetMaybeDefault, AssignmentTargetProperty};
type WriteTarget = (String, ValueMutationPath);
pub(in super::super) fn targets(target: &AssignmentTarget<'_>) -> Vec<WriteTarget> {
    match target {
        AssignmentTarget::AssignmentTargetIdentifier(_) => assignment_target_names(target)
            .into_iter()
            .map(|name| (name, Vec::new()))
            .collect(),
        AssignmentTarget::StaticMemberExpression(member) => {
            member_target(&member.object, Some(member.property.name.to_string()))
                .into_iter()
                .collect()
        }
        AssignmentTarget::ComputedMemberExpression(member) => {
            member_target(&member.object, static_key(&member.expression))
                .into_iter()
                .collect()
        }
        AssignmentTarget::ArrayAssignmentTarget(array) => array
            .elements
            .iter()
            .flatten()
            .flat_map(maybe_default)
            .chain(array.rest.iter().flat_map(|rest| targets(&rest.target)))
            .collect(),
        AssignmentTarget::ObjectAssignmentTarget(object) => object
            .properties
            .iter()
            .flat_map(|property| match property {
                AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(property) => {
                    vec![(property.binding.name.to_string(), Vec::new())]
                }
                AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
                    maybe_default(&property.binding)
                }
            })
            .chain(object.rest.iter().flat_map(|rest| targets(&rest.target)))
            .collect(),
        _ => Vec::new(),
    }
}
fn maybe_default(target: &AssignmentTargetMaybeDefault<'_>) -> Vec<WriteTarget> {
    if let AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(default) = target {
        return targets(&default.binding);
    }
    target.as_assignment_target().map_or_else(Vec::new, targets)
}
fn member_target(object: &Expression<'_>, property: Option<String>) -> Option<WriteTarget> {
    let (base, mut path) = expression_path(object)?;
    path.push(property);
    let name = std::iter::once(base)
        .chain(
            path.iter()
                .map(|part| part.as_deref().unwrap_or("*").to_string()),
        )
        .collect::<Vec<_>>()
        .join(".");
    Some((name, path))
}
fn expression_path(expression: &Expression<'_>) -> Option<(String, ValueMutationPath)> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
        Expression::Identifier(identifier) => Some((identifier.name.to_string(), Vec::new())),
        Expression::StaticMemberExpression(member) => {
            let (base, mut path) = expression_path(&member.object)?;
            path.push(Some(member.property.name.to_string()));
            Some((base, path))
        }
        Expression::ComputedMemberExpression(member) => {
            let (base, mut path) = expression_path(&member.object)?;
            path.push(static_key(&member.expression));
            Some((base, path))
        }
        _ => None,
    }
}
pub(super) fn static_key(expression: &Expression<'_>) -> Option<String> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
        Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        Expression::NumericLiteral(literal) => Some(literal.value.to_string()),
        _ => None,
    }
}

pub(super) fn property_key(key: &oxc_ast::ast::PropertyKey<'_>) -> Option<String> {
    if let oxc_ast::ast::PropertyKey::NumericLiteral(literal) = key {
        Some(literal.value.to_string())
    } else {
        crate::codebase::ts_source::static_property_key_name(key).map(str::to_string)
    }
}

/// Inherit later sibling ranges, never an enclosing default or assignment RHS.
pub(in super::super) fn record_early_targets(
    collector: &mut ImportCollector,
    target: &AssignmentTarget<'_>,
) {
    if collector.prior_member_mutations.is_empty() || !collector.has_early_assignment_target(target)
    {
        return;
    }
    walk_early_targets(collector, target, &mut Vec::new());
}
fn walk_early_targets(
    collector: &mut ImportCollector,
    target: &AssignmentTarget<'_>,
    later: &mut Vec<(u32, u32)>,
) {
    match target {
        AssignmentTarget::ArrayAssignmentTarget(array) => {
            let last = array.elements.iter().rposition(Option::is_some);
            for (index, element) in array.elements.iter().enumerate() {
                if let Some(element) = element {
                    let end =
                        (Some(index) != last || array.rest.is_some()).then_some(array.span.end);
                    record_early_default(collector, element, end, later);
                }
            }
            if let Some(rest) = &array.rest {
                walk_early_targets(collector, &rest.target, later);
            }
        }
        AssignmentTarget::ObjectAssignmentTarget(object) => {
            for (index, property) in object.properties.iter().enumerate() {
                if let AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) =
                    property
                {
                    let end = (index + 1 < object.properties.len() || object.rest.is_some())
                        .then_some(object.span.end);
                    record_early_default(collector, &property.binding, end, later);
                }
            }
            if let Some(rest) = &object.rest {
                walk_early_targets(collector, &rest.target, later);
            }
        }
        AssignmentTarget::StaticMemberExpression(_)
        | AssignmentTarget::ComputedMemberExpression(_) => {
            for (name, path) in targets(target) {
                collector.record_early_assignment(&name, &path, later);
            }
        }
        _ => {}
    }
}
fn record_early_default(
    collector: &mut ImportCollector,
    target: &AssignmentTargetMaybeDefault<'_>,
    end: Option<u32>,
    later: &mut Vec<(u32, u32)>,
) {
    let length = later.len();
    // The target's own default completes before its write, even though its
    // binding is printed first. Enclosing defaults are excluded by recursion.
    if let Some(end) = end {
        #[cfg(feature = "test-instrumentation")]
        if later.len() == later.capacity() {
            collector.value_flow.early_assignment_stack_growths += 1;
        }
        later.push((target.span().end, end));
    }
    let binding = match target {
        AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(default) => &default.binding,
        _ => target.as_assignment_target().expect("assignment target"),
    };
    walk_early_targets(collector, binding, later);
    later.truncate(length);
}
