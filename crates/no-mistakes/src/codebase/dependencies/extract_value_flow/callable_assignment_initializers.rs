impl ImportCollector {
    fn record_callable_initializer(
        &mut self,
        declarator: &VariableDeclarator<'_>,
        immutable: bool,
        init: &Expression<'_>,
    ) {
        if !immutable {
            for name in binding_names(&declarator.id) {
                if self.mutable_module_bindings.contains(&name)
                    && self.callee_binding_scope(&name) == Some(0)
                {
                    self.value_flow
                        .callable_assignments
                        .push(CallableAssignment {
                            scope: 0,
                            name,
                            completed_at: init.span().end,
                            callable: None,
                            owner: self.current_function_id(),
                            region_start: self.replacement_region.map(|region| region.start),
                            guaranteed: false,
                        });
                }
            }
        }
    }
}
