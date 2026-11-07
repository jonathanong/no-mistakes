#[derive(Default)]
struct CounterWrites {
    bindings: FxHashSet<String>,
    writes: FxHashSet<String>,
    opaque: bool,
}

impl<'a> Visit<'a> for CounterWrites {
    fn visit_binding_identifier(&mut self, identifier: &oxc_ast::ast::BindingIdentifier<'a>) {
        self.bindings.insert(identifier.name.to_string());
    }

    fn visit_assignment_target(&mut self, target: &AssignmentTarget<'a>) {
        self.writes.extend(
            extract_value_flow::write_targets::targets(target)
                .into_iter()
                .map(|(name, _, _)| name),
        );
        walk::walk_assignment_target(self, target);
    }

    fn visit_update_expression(&mut self, update: &oxc_ast::ast::UpdateExpression<'a>) {
        if let oxc_ast::ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(identifier) =
            &update.argument
        {
            self.writes.insert(identifier.name.to_string());
        }
        walk::walk_update_expression(self, update);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.opaque |= simple_callee_name(&call.callee).as_deref() == Some("eval");
        walk::walk_call_expression(self, call);
    }
}
