use super::{resolve::functions::reassigned::for_each_assigned_name, ScopeVisitor};
use oxc_ast::ast::{AssignmentTarget, AssignmentTargetMaybeDefault, AssignmentTargetProperty};
use oxc_ast_visit::Visit;

pub(super) fn visit_target<'a>(visitor: &mut ScopeVisitor<'a>, target: &AssignmentTarget<'a>) {
    match target {
        AssignmentTarget::ArrayAssignmentTarget(array) => {
            for element in array.elements.iter().flatten() {
                visit_default(visitor, element);
            }
            if let Some(rest) = &array.rest {
                visit_target(visitor, &rest.target);
            }
        }
        AssignmentTarget::ObjectAssignmentTarget(object) => {
            for property in &object.properties {
                match property {
                    AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(property) => {
                        if let Some(init) = &property.init {
                            visitor.visit_expression(init);
                        }
                        visitor.mark_dynamic(property.binding.name.as_str());
                    }
                    AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
                        visitor.visit_property_key(&property.name);
                        visit_default(visitor, &property.binding);
                    }
                }
            }
            if let Some(rest) = &object.rest {
                visit_target(visitor, &rest.target);
            }
        }
        _ => {
            visitor.visit_assignment_target(target);
            for_each_assigned_name(target, &mut |name| visitor.mark_dynamic(name));
        }
    }
}

fn visit_default<'a>(visitor: &mut ScopeVisitor<'a>, target: &AssignmentTargetMaybeDefault<'a>) {
    if let AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(default) = target {
        let mut writes_binding = false;
        for_each_assigned_name(&default.binding, &mut |_| writes_binding = true);
        // Member references are evaluated before their default; nested patterns
        // and identifier writes follow evaluation of the default value.
        if writes_binding
            || matches!(
                default.binding,
                AssignmentTarget::ArrayAssignmentTarget(_)
                    | AssignmentTarget::ObjectAssignmentTarget(_)
            )
        {
            visitor.visit_expression(&default.init);
            visit_target(visitor, &default.binding);
        } else {
            visit_target(visitor, &default.binding);
            visitor.visit_expression(&default.init);
        }
    } else {
        visit_target(visitor, target.to_assignment_target());
    }
}
