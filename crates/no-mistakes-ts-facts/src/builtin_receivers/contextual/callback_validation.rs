use oxc_ast::ast::*;
use oxc_ast_visit::{walk, Visit};

pub(super) fn arrow_has_binding(body: &ArrowFunctionBody<'_>, name: &str) -> bool {
    let mut scan = BindingScan { name, found: false };
    if let ArrowFunctionBody::FunctionBody(body) = body {
        scan.visit_function_body(body);
    }
    scan.found
}

pub(super) fn body_has_binding(body: &FunctionBody<'_>, name: &str) -> bool {
    let mut scan = BindingScan { name, found: false };
    scan.visit_function_body(body);
    scan.found
}

struct BindingScan<'a> {
    name: &'a str,
    found: bool,
}

impl<'a> Visit<'a> for BindingScan<'_> {
    fn visit_binding_identifier(&mut self, identifier: &BindingIdentifier<'a>) {
        self.found |= identifier.name == self.name;
    }
}

pub(super) fn arrow_has_write(body: &ArrowFunctionBody<'_>, name: &str) -> bool {
    let mut writes = ParameterWrites { name, found: false };
    if let Some(expression) = body.as_expression() {
        writes.visit_expression(expression);
    } else if let ArrowFunctionBody::FunctionBody(body) = body {
        writes.visit_function_body(body);
    }
    writes.found
}

pub(super) fn body_has_write(body: &FunctionBody<'_>, name: &str) -> bool {
    let mut writes = ParameterWrites { name, found: false };
    writes.visit_function_body(body);
    writes.found
}

struct ParameterWrites<'a> {
    name: &'a str,
    found: bool,
}

impl<'a> Visit<'a> for ParameterWrites<'_> {
    fn visit_assignment_target(&mut self, target: &AssignmentTarget<'a>) {
        self.found |= matches!(
            target,
            AssignmentTarget::AssignmentTargetIdentifier(identifier)
                if identifier.name == self.name
        );
        walk::walk_assignment_target(self, target);
    }

    fn visit_update_expression(&mut self, update: &UpdateExpression<'a>) {
        self.found |= matches!(
            &update.argument,
            SimpleAssignmentTarget::AssignmentTargetIdentifier(identifier)
                if identifier.name == self.name
        );
        walk::walk_update_expression(self, update);
    }

    fn visit_assignment_target_property_identifier(
        &mut self,
        property: &AssignmentTargetPropertyIdentifier<'a>,
    ) {
        self.found |= property.binding.name == self.name;
        walk::walk_assignment_target_property_identifier(self, property);
    }
}

pub(super) fn string_result_method(method: &str) -> bool {
    matches!(
        method,
        "trim"
            | "trimStart"
            | "trimEnd"
            | "padStart"
            | "padEnd"
            | "slice"
            | "substring"
            | "toLowerCase"
            | "toUpperCase"
            | "toString"
            | "valueOf"
    )
}
