fn register_helper_assignment_alias(
    assignment: &oxc::ast::ast::AssignmentExpression<'_>,
    bindings: &mut RouteHelperBindings,
) {
    if assignment.operator != oxc::ast::ast::AssignmentOperator::Assign {
        return;
    }
    let oxc::ast::ast::AssignmentTarget::AssignmentTargetIdentifier(ident) = &assignment.left else {
        return;
    };
    let target = helper_alias_target(&assignment.right, bindings);
    remove_shadowed_helper_name(ident.name.as_str(), bindings);
    if let Some(target) = target {
        bindings.identifiers.insert(ident.name.to_string());
        bindings.aliases.insert(ident.name.to_string(), target);
    }
}

