impl ImportCollector {
    fn record_callable_initializer(
        &mut self,
        declarator: &VariableDeclarator<'_>,
        immutable: bool,
        init: &Expression<'_>,
    ) {
        if !immutable {
            for name in binding_names(&declarator.id) {
                if self.is_mutable_module_binding(&name)
                    && self.callee_binding_scope(&name) == Some(0)
                {
                    let owner = self.current_function_id();
                    self.callable_assignment_state
                        .as_mut()
                        .expect("mutable module state")
                        .assignments
                        .push(CallableAssignment {
                            scope: 0,
                            name,
                            completed_at: init.span().end,
                            callable: None,
                            owner,
                            region_start: self.replacement_region.map(|region| region.start),
                            guaranteed: false,
                        });
                }
            }
        }
    }
}
