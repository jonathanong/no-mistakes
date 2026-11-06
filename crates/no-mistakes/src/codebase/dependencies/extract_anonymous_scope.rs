impl ImportCollector {
    fn push_anonymous_function_scope(&mut self, id: CallableId) {
        self.anonymous_scope_count += 1;
        let name = format!("<anonymous:{}>", self.anonymous_scope_count);
        let scope = self
            .function_stack
            .last()
            .map(|parent| format!("{parent}/{name}"))
            .unwrap_or(name);
        self.known_function_scopes.insert(scope.clone());
        self.callable_scopes.insert(scope.clone());
        self.callable_scope_ids.insert((id, scope.clone()));
        // Module callbacks need the same synthetic edge as nested callbacks:
        // a top-level IIFE/callback is available to file-root call analysis,
        // while execution-sensitive projections still distinguish the
        // synthetic invocation through `is_callback`.
        self.function_calls.push(FunctionCall {
            caller: self.function_stack.last().cloned(),
            caller_id: self.current_function_id(),
            syntactic_caller: self.current_syntactic_caller(),
            callee: scope.clone(),
            line: 0,
            offset: id.0,
            is_callback: true,
            invocation: InvocationKind::Callback,
            target_identity: CallTargetIdentity::RepositoryFunction,
            callee_binding_scope: None,
            static_arg: None,
            static_cwd: None,
        });
        self.function_stack.push(scope);
        self.function_id_stack.push((id, false));
        self.var_scope_stack.push(self.local_stack.len());
        self.local_stack.push(fx_set());
        let lexical_scope_id = self.next_lexical_scope_id;
        self.lexical_scope_parents
            .insert(lexical_scope_id, self.lexical_scope_ids.last().copied());
        self.lexical_scope_ids.push(lexical_scope_id);
        self.next_lexical_scope_id += 1;
        self.type_local_stack.push(fx_set());
        self.type_parameter_stack.push(fx_set());
    }

}
