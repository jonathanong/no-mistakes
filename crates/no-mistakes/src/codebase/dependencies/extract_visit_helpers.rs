impl ImportCollector {
    fn call_target_identity(&self, callee: &str) -> CallTargetIdentity {
        let binding = callee
            .split_once('.')
            .map_or(callee, |(binding, _)| binding);
        if self.local_binding_shadows(binding) {
            return if self.has_local_function_scope(callee) {
                CallTargetIdentity::RepositoryFunction
            } else {
                CallTargetIdentity::Unknown
            };
        }
        if self.imported_bindings.contains(binding)
            || self.predeclared_imported_bindings.contains(binding)
        {
            return CallTargetIdentity::ModuleExport;
        }
        if callee == binding || matches!(binding, "globalThis" | "window" | "self" | "global") {
            return CallTargetIdentity::Global;
        }
        CallTargetIdentity::Unknown
    }

    fn current_function(&self) -> Option<String> {
        self.function_stack.last().cloned()
    }

    fn push_value_symbol_reference(&mut self, name: String) {
        let caller = self.current_function();
        if self.callee_shadows_import(&name) {
            return;
        }
        let callee_binding_scope = self.callee_binding_scope(&name);
        self.symbol_references.push(FunctionCall {
            caller,
            syntactic_caller: self.current_syntactic_caller(),
            callee: name,
            line: 0,
            offset: 0,
            is_callback: false,
            invocation: InvocationKind::Call,
            target_identity: CallTargetIdentity::Unknown,
            callee_binding_scope,
            static_arg: None,
            static_cwd: None,
        });
    }

    fn add_formal_parameters(&mut self, params: &FormalParameters<'_>) {
        for param in &params.items {
            self.add_binding_names(&param.pattern);
        }
        if let Some(rest) = &params.rest {
            self.add_binding_names(&rest.rest.argument);
        }
    }

    fn add_var_binding_names(&mut self, pattern: &BindingPattern<'_>) {
        for name in binding_names(pattern) {
            self.add_var_binding_name(&name);
        }
    }

    // Kept as the narrower historical helper for its direct coverage test.
    fn add_var_binding_name(&mut self, name: &str) {
        let Some(index) = self
            .function_scope_stack
            .last()
            .copied()
            .or_else(|| (!self.local_stack.is_empty()).then_some(0))
        else {
            return;
        };
        let Some(scope) = self.local_stack.get_mut(index) else {
            return;
        };
        scope.insert(name.to_string());
    }

    fn add_binding_names(&mut self, pattern: &BindingPattern<'_>) {
        let Some(scope) = self.local_stack.last_mut() else {
            return;
        };
        for name in binding_names(pattern) {
            scope.insert(name);
        }
    }

    fn add_binding_name(&mut self, name: &str) {
        let Some(scope) = self.local_stack.last_mut() else {
            return;
        };
        scope.insert(name.to_string());
    }

    fn add_type_binding_name(&mut self, name: &str) {
        if self.type_local_stack.is_empty() {
            self.type_local_stack.push(HashSet::new());
        }
        if let Some(scope) = self.type_local_stack.last_mut() {
            scope.insert(name.to_string());
        }
    }

    fn local_binding_shadows(&self, name: &str) -> bool {
        self.local_stack
            .iter()
            .rev()
            .any(|scope| scope.contains(name))
    }

    fn current_lexical_scope_id(&self) -> usize {
        *self
            .lexical_scope_ids
            .last()
            .expect("collector always establishes the program lexical scope")
    }

    fn callee_binding_scope(&self, callee: &str) -> Option<usize> {
        let binding = callee.split_once('.').map_or(callee, |(binding, _)| binding);
        self.local_stack
            .iter()
            .rposition(|scope| scope.contains(binding))
            .map(|depth| self.lexical_scope_ids[depth])
    }

    fn record_callable_binding(&mut self, name: &str) {
        self.callable_binding_ids
            .insert((self.current_lexical_scope_id(), name.to_string()));
    }

    fn callee_shadows_import(&self, callee: &str) -> bool {
        let binding = callee
            .split_once('.')
            .map_or(callee, |(binding, _)| binding);
        self.local_binding_shadows(binding)
            && (self.imported_bindings.contains(binding)
                || self.predeclared_imported_bindings.contains(binding))
    }

    fn has_local_function_scope(&self, callee: &str) -> bool {
        let binding = callee
            .split_once('.')
            .map_or(callee, |(binding, _)| binding);
        let Some(binding_scope) = self.callee_binding_scope(callee) else {
            return false;
        };
        let callable_alias = self.callable_aliases.iter().any(|alias| {
            alias.alias.binding_scope == binding_scope && alias.alias.local == binding
        });
        if (!self
            .callable_binding_ids
            .contains(&(binding_scope, binding.to_string()))
            && !callable_alias)
            || self
                .reassigned_callable_binding_ids
                .contains(&(binding_scope, binding.to_string()))
        {
            return false;
        }
        if callable_alias {
            return true;
        }
        // `api/run` can mean either an aggregate member or a lexical nested
        // function. A declared `function api` owns the latter spelling, so a
        // static `api.run()` must not be guessed as an aggregate dispatch.
        if callee.contains('.') && self.callable_scopes.contains(binding) {
            return false;
        }
        let Some(caller) = self.current_function() else {
            return self.callable_scopes.contains(binding)
                || self
                    .known_function_scopes
                    .contains(&binding.replace('.', "/"));
        };
        let mut scope = caller.as_str();
        loop {
            let candidate = format!("{scope}/{binding}");
            let member_candidate = format!("{scope}/{}", binding.replace('.', "/"));
            if (self.callable_scopes.contains(&candidate)
                && !self.reassigned_callable_scopes.contains(&candidate))
                || (self.callable_scopes.contains(&member_candidate)
                    && !self.reassigned_callable_scopes.contains(&member_candidate))
            {
                return true;
            }
            let Some((parent, _)) = scope.rsplit_once('/') else {
                return (self.callable_scopes.contains(binding)
                    && !self.reassigned_callable_scopes.contains(binding))
                    || (self.callable_scopes.contains(&binding.replace('.', "/"))
                        && !self
                            .reassigned_callable_scopes
                            .contains(&binding.replace('.', "/")));
            };
            scope = parent;
        }
    }
}
