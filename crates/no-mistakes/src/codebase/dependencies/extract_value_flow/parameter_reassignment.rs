impl ImportCollector {
    fn note_parameter_reassignment(&mut self, assignment: &AssignmentExpression<'_>) {
        if !self.collect_value_flow || !assignment.operator.is_assign() {
            return;
        }
        let Some(name) = mutable_root_name(&assignment.left) else {
            return;
        };
        let Some(scope) = self.callee_binding_scope(name) else {
            return;
        };
        let bare = self
            .value_parameter_indexes
            .get(&scope)
            .and_then(|names| names.get(name))
            .is_some_and(|indexes| {
                indexes.iter().any(|index| {
                    self.value_flow
                        .parameters
                        .get(*index)
                        .is_some_and(|parameter| parameter.projection.is_empty())
                })
            });
        if !bare {
            return;
        }
        self.parameter_reassignments.push((
            scope,
            name.to_string(),
            self.value_expression(&assignment.right),
        ));
    }
}
