impl CallableFileIndex {
    fn alias_live_at(
        &self,
        alias: &IndexedAlias,
        alias_scope: usize,
        offset: u32,
        call_binding_scope: Option<usize>,
        caller_id: Option<crate::codebase::dependencies::extract::CallableId>,
    ) -> bool {
        binding_live_at(BindingLivenessQuery {
            declared_at: alias.declared_at,
            invalidated_at: alias.invalidated_at,
            call_offset: offset,
            binding_scope: alias_scope,
            call_binding_scope,
            caller_id,
            invocation_offsets: &self.invocation_offsets,
            lexical_parents: &self.lexical_scope_parents,
        })
    }

    fn target_binding_live(
        &self,
        mut scope: usize,
        name: &str,
        offset: u32,
        call_binding_scope: Option<usize>,
        caller_id: Option<crate::codebase::dependencies::extract::CallableId>,
    ) -> bool {
        let binding = name
            .split_once('.')
            .map_or(name, |(binding, _)| binding);
        loop {
            if let Some(declared_at) = self.binding_declared_at.get(&(scope, binding.to_string())) {
                return binding_live_at(BindingLivenessQuery {
                    declared_at: *declared_at,
                    invalidated_at: None,
                    call_offset: offset,
                    binding_scope: scope,
                    call_binding_scope,
                    caller_id,
                    invocation_offsets: &self.invocation_offsets,
                    lexical_parents: &self.lexical_scope_parents,
                });
            }
            if self
                .callable_bindings
                .contains_key(&(scope, binding.to_string()))
            {
                return true;
            }
            match self.lexical_scope_parents.get(&scope).copied().flatten() {
                Some(parent) => scope = parent,
                None => return true,
            }
        }
    }
}
