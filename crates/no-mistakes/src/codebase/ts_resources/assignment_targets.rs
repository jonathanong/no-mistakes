use oxc_ast::ast::{
    AssignmentTarget, AssignmentTargetMaybeDefault, AssignmentTargetProperty, AssignmentTargetRest,
};

pub(super) fn assignment_target_names(target: &AssignmentTarget<'_>) -> Vec<String> {
    match target {
        AssignmentTarget::AssignmentTargetIdentifier(id) => vec![id.name.to_string()],
        AssignmentTarget::ArrayAssignmentTarget(array) => {
            let mut names = array
                .elements
                .iter()
                .flatten()
                .flat_map(maybe_default_names)
                .collect();
            extend_rest_names(&mut names, array.rest.as_deref());
            names
        }
        AssignmentTarget::ObjectAssignmentTarget(object) => {
            let mut names = object.properties.iter().flat_map(property_names).collect();
            extend_rest_names(&mut names, object.rest.as_deref());
            names
        }
        _ => Vec::new(),
    }
}

fn maybe_default_names(target: &AssignmentTargetMaybeDefault<'_>) -> Vec<String> {
    match target {
        AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(default) => {
            assignment_target_names(&default.binding)
        }
        AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(id) => vec![id.name.to_string()],
        AssignmentTargetMaybeDefault::ArrayAssignmentTarget(array) => {
            let mut names = array
                .elements
                .iter()
                .flatten()
                .flat_map(maybe_default_names)
                .collect();
            extend_rest_names(&mut names, array.rest.as_deref());
            names
        }
        AssignmentTargetMaybeDefault::ObjectAssignmentTarget(object) => {
            let mut names = object.properties.iter().flat_map(property_names).collect();
            extend_rest_names(&mut names, object.rest.as_deref());
            names
        }
        _ => Vec::new(),
    }
}

fn property_names(property: &AssignmentTargetProperty<'_>) -> Vec<String> {
    match property {
        AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(property) => {
            vec![property.binding.name.to_string()]
        }
        AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
            maybe_default_names(&property.binding)
        }
    }
}

fn extend_rest_names(names: &mut Vec<String>, rest: Option<&AssignmentTargetRest<'_>>) {
    if let Some(rest) = rest {
        names.extend(assignment_target_names(&rest.target));
    }
}
