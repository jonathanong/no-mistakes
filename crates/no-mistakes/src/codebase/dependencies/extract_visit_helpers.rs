impl ImportCollector {
    fn call_target_identity(&self, callee: &str) -> CallTargetIdentity {
        let binding = callee
            .split_once('.')
            .map_or(callee, |(binding, _)| binding);
        if self.local_binding_shadows(binding) {
            return self
                .has_local_function_scope(callee)
                .then_some(CallTargetIdentity::RepositoryFunction)
                .unwrap_or(CallTargetIdentity::Unknown);
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
        self.symbol_references.push(FunctionCall {
            caller,
            callee: name,
            line: 0,
            offset: 0,
            is_callback: false,
            invocation: InvocationKind::Call,
            target_identity: CallTargetIdentity::Unknown,
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
            if self.callable_scopes.contains(&candidate)
                || self.callable_scopes.contains(&member_candidate)
            {
                return true;
            }
            let Some((parent, _)) = scope.rsplit_once('/') else {
                return self.callable_scopes.contains(binding)
                    || self.callable_scopes.contains(&binding.replace('.', "/"));
            };
            scope = parent;
        }
    }
}
