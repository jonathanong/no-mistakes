use super::{
    client_module_expr, is_client_method_name, modules::is_client_factory_member, ClientHttpVisitor,
};
use oxc_ast::ast::{
    AssignmentExpression, AssignmentTarget, AssignmentTargetMaybeDefault, AssignmentTargetProperty,
    AssignmentTargetRest,
};

impl ClientHttpVisitor<'_> {
    pub(super) fn update_client_assignment(&mut self, assignment: &AssignmentExpression<'_>) {
        if self.client_factory_method_expr(&assignment.right) {
            for name in assignment_target_names(&assignment.left) {
                self.assign_client_factory_callee_name(name);
            }
        } else if self.client_object_method_expr(&assignment.right) {
            for name in assignment_target_names(&assignment.left) {
                self.assign_client_callee_name(name);
            }
        } else if client_module_expr(&assignment.right) || self.client_expr(&assignment.right) {
            self.assign_client_bindings_from_target(&assignment.left);
        } else {
            for name in assignment_target_names(&assignment.left) {
                self.assign_shadow_name(name);
            }
        }
    }

    pub(super) fn assign_client_bindings_from_target(&mut self, target: &AssignmentTarget<'_>) {
        match target {
            AssignmentTarget::ObjectAssignmentTarget(object) => {
                self.assign_client_bindings_from_object_properties(&object.properties);
                if let Some(rest) = &object.rest {
                    self.assign_client_bindings_from_target(&rest.target);
                }
            }
            AssignmentTarget::ArrayAssignmentTarget(_) => {
                for name in assignment_target_names(target) {
                    self.assign_shadow_name(name);
                }
            }
            _ => {
                for name in assignment_target_names(target) {
                    self.assign_client_name(name);
                }
            }
        }
    }

    fn assign_client_bindings_from_object_properties(
        &mut self,
        properties: &[AssignmentTargetProperty<'_>],
    ) {
        for prop in properties {
            self.assign_client_bindings_from_object_property(prop);
        }
    }

    fn assign_client_bindings_from_object_property(&mut self, prop: &AssignmentTargetProperty<'_>) {
        match prop {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(prop) => {
                self.assign_binding_for_property_name(prop.binding.name.as_str());
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(prop) => {
                let names = assignment_target_maybe_default_names(&prop.binding);
                if prop
                    .name
                    .static_name()
                    .is_some_and(|key| is_client_method_name(key.as_ref()))
                {
                    self.assign_client_callee_names(names);
                } else if prop
                    .name
                    .static_name()
                    .is_some_and(|key| is_client_factory_member(&key))
                {
                    self.assign_client_factory_callee_names(names);
                } else {
                    self.assign_shadow_names(names);
                }
            }
        }
    }

    fn assign_binding_for_property_name(&mut self, name: &str) {
        if is_client_method_name(name) {
            self.assign_client_callee_name(name.to_string());
        } else if is_client_factory_member(name) {
            self.assign_client_factory_callee_name(name.to_string());
        } else {
            self.assign_shadow_name(name.to_string());
        }
    }

    fn assign_client_callee_names(&mut self, names: Vec<String>) {
        for name in names {
            self.assign_client_callee_name(name);
        }
    }

    fn assign_client_factory_callee_names(&mut self, names: Vec<String>) {
        for name in names {
            self.assign_client_factory_callee_name(name);
        }
    }

    fn assign_shadow_names(&mut self, names: Vec<String>) {
        for name in names {
            self.assign_shadow_name(name);
        }
    }
}

pub(super) fn assignment_target_names(target: &AssignmentTarget<'_>) -> Vec<String> {
    match target {
        AssignmentTarget::AssignmentTargetIdentifier(id) => vec![id.name.to_string()],
        AssignmentTarget::ArrayAssignmentTarget(array) => {
            let rest = array.rest.as_deref().map(assignment_rest_target);
            array_assignment_target_names(array.elements.iter().flatten(), rest)
        }
        AssignmentTarget::ObjectAssignmentTarget(object) => {
            let rest = object.rest.as_deref().map(assignment_rest_target);
            object_assignment_target_names(&object.properties, rest)
        }
        _ => Vec::new(),
    }
}

fn assignment_target_maybe_default_names(target: &AssignmentTargetMaybeDefault<'_>) -> Vec<String> {
    match target {
        AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(default) => {
            assignment_target_names(&default.binding)
        }
        AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(id) => vec![id.name.to_string()],
        AssignmentTargetMaybeDefault::ArrayAssignmentTarget(array) => {
            let rest = array.rest.as_deref().map(assignment_rest_target);
            array_assignment_target_names(array.elements.iter().flatten(), rest)
        }
        AssignmentTargetMaybeDefault::ObjectAssignmentTarget(object) => {
            let rest = object.rest.as_deref().map(assignment_rest_target);
            object_assignment_target_names(&object.properties, rest)
        }
        _ => Vec::new(),
    }
}

fn array_assignment_target_names<'a>(
    elements: impl Iterator<Item = &'a AssignmentTargetMaybeDefault<'a>>,
    rest: Option<&'a AssignmentTarget<'a>>,
) -> Vec<String> {
    let mut names = elements
        .flat_map(assignment_target_maybe_default_names)
        .collect();
    extend_assignment_rest_names(&mut names, rest);
    names
}

fn object_assignment_target_names(
    properties: &[AssignmentTargetProperty<'_>],
    rest: Option<&AssignmentTarget<'_>>,
) -> Vec<String> {
    let mut names = properties
        .iter()
        .flat_map(assignment_property_names)
        .collect();
    extend_assignment_rest_names(&mut names, rest);
    names
}

fn assignment_property_names(prop: &AssignmentTargetProperty<'_>) -> Vec<String> {
    match prop {
        AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(prop) => {
            vec![prop.binding.name.to_string()]
        }
        AssignmentTargetProperty::AssignmentTargetPropertyProperty(prop) => {
            assignment_target_maybe_default_names(&prop.binding)
        }
    }
}

fn extend_assignment_rest_names(names: &mut Vec<String>, rest: Option<&AssignmentTarget<'_>>) {
    if let Some(rest) = rest {
        names.extend(assignment_target_names(rest));
    }
}

fn assignment_rest_target<'a>(rest: &'a AssignmentTargetRest<'a>) -> &'a AssignmentTarget<'a> {
    &rest.target
}
