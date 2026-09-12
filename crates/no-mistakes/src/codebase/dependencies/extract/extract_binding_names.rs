use oxc_ast::ast::{
    AssignmentTarget, AssignmentTargetMaybeDefault, AssignmentTargetProperty, BindingPattern,
};

#[inline(never)]
pub(super) fn binding_names(pattern: &BindingPattern<'_>) -> Vec<String> {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => vec![identifier.name.to_string()],
        BindingPattern::ObjectPattern(object) => object
            .properties
            .iter()
            .flat_map(|property| binding_names(&property.value))
            .collect(),
        BindingPattern::ArrayPattern(array) => array
            .elements
            .iter()
            .flatten()
            .flat_map(binding_names)
            .collect(),
        BindingPattern::AssignmentPattern(assignment) => binding_names(&assignment.left),
    }
}

#[inline(never)]
pub(super) fn assignment_target_names(target: &AssignmentTarget<'_>) -> Vec<String> {
    match target {
        AssignmentTarget::AssignmentTargetIdentifier(identifier) => {
            vec![identifier.name.to_string()]
        }
        AssignmentTarget::ArrayAssignmentTarget(array) => array
            .elements
            .iter()
            .flatten()
            .flat_map(assignment_target_maybe_default_names)
            .chain(
                array
                    .rest
                    .iter()
                    .flat_map(|rest| assignment_target_names(&rest.target)),
            )
            .collect(),
        AssignmentTarget::ObjectAssignmentTarget(object) => object
            .properties
            .iter()
            .flat_map(|property| match property {
                AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(property) => {
                    vec![property.binding.name.to_string()]
                }
                AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
                    assignment_target_maybe_default_names(&property.binding)
                }
            })
            .chain(
                object
                    .rest
                    .iter()
                    .flat_map(|rest| assignment_target_names(&rest.target)),
            )
            .collect(),
        AssignmentTarget::StaticMemberExpression(member) => {
            super::simple_static_member_name(member)
                .into_iter()
                .collect()
        }
        AssignmentTarget::ComputedMemberExpression(member) => {
            super::simple_computed_member_name(member)
                .into_iter()
                .collect()
        }
        _ => Vec::new(),
    }
}

#[inline(never)]
pub(super) fn assignment_target_maybe_default_names(
    target: &AssignmentTargetMaybeDefault<'_>,
) -> Vec<String> {
    match target {
        AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(target) => {
            assignment_target_names(&target.binding)
        }
        AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(identifier) => {
            vec![identifier.name.to_string()]
        }
        AssignmentTargetMaybeDefault::ArrayAssignmentTarget(array) => array
            .elements
            .iter()
            .flatten()
            .flat_map(assignment_target_maybe_default_names)
            .chain(
                array
                    .rest
                    .iter()
                    .flat_map(|rest| assignment_target_names(&rest.target)),
            )
            .collect(),
        AssignmentTargetMaybeDefault::ObjectAssignmentTarget(object) => object
            .properties
            .iter()
            .flat_map(|property| match property {
                AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(property) => {
                    vec![property.binding.name.to_string()]
                }
                AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
                    assignment_target_maybe_default_names(&property.binding)
                }
            })
            .chain(
                object
                    .rest
                    .iter()
                    .flat_map(|rest| assignment_target_names(&rest.target)),
            )
            .collect(),
        _ => Vec::new(),
    }
}
