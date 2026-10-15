use super::*;
use oxc_ast::ast::{AssignmentTargetMaybeDefault, AssignmentTargetProperty};
type WriteTarget = (String, ValueMutationPath, u32);
pub(in super::super) fn targets(target: &AssignmentTarget<'_>) -> Vec<WriteTarget> {
    match target {
        AssignmentTarget::AssignmentTargetIdentifier(_) => assignment_target_names(target)
            .into_iter()
            .map(|name| (name, Vec::new(), target.span().start))
            .collect(),
        AssignmentTarget::StaticMemberExpression(member) => member_target(
            &member.object,
            Some(member.property.name.to_string()),
            target.span().start,
        )
        .into_iter()
        .collect(),
        AssignmentTarget::ComputedMemberExpression(member) => member_target(
            &member.object,
            static_key(&member.expression),
            target.span().start,
        )
        .into_iter()
        .collect(),
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
                    vec![(
                        property.binding.name.to_string(),
                        Vec::new(),
                        property.binding.span.start,
                    )]
                }
                AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
                    maybe_default(&property.binding)
                }
            })
            .chain(object.rest.iter().flat_map(|rest| targets(&rest.target)))
            .collect(),
        AssignmentTarget::TSAsExpression(value) => {
            wrapped_target(&value.expression, target.span().start)
        }
        AssignmentTarget::TSSatisfiesExpression(value) => {
            wrapped_target(&value.expression, target.span().start)
        }
        AssignmentTarget::TSNonNullExpression(value) => {
            wrapped_target(&value.expression, target.span().start)
        }
        AssignmentTarget::TSTypeAssertion(value) => {
            wrapped_target(&value.expression, target.span().start)
        }
        _ => Vec::new(),
    }
}
fn wrapped_target(expression: &Expression<'_>, start: u32) -> Vec<WriteTarget> {
    if let Expression::Identifier(identifier) =
        crate::codebase::ts_source::unwrap_ts_wrappers(expression)
    {
        vec![(identifier.name.to_string(), Vec::new(), start)]
    } else {
        Vec::new()
    }
}
fn maybe_default(target: &AssignmentTargetMaybeDefault<'_>) -> Vec<WriteTarget> {
    if let AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(default) = target {
        return targets(&default.binding);
    }
    target.as_assignment_target().map_or_else(Vec::new, targets)
}
fn member_target(
    object: &Expression<'_>,
    property: Option<String>,
    start: u32,
) -> Option<WriteTarget> {
    let (base, mut path) = expression_path(object)?;
    path.push(property);
    let name = std::iter::once(base)
        .chain(
            path.iter()
                .map(|part| part.as_deref().unwrap_or("*").to_string()),
        )
        .collect::<Vec<_>>()
        .join(".");
    Some((name, path, start))
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
