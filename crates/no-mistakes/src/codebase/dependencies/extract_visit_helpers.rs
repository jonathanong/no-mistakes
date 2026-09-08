impl ImportCollector {
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

    fn add_function_binding_names(&mut self, pattern: &BindingPattern<'_>) {
        // `var` is function-scoped. At program level the root lexical scope
        // is its function-equivalent owner, so block-local `var` declarations
        // still shadow globals after the block.
        let index = self.function_scope_stack.last().copied().unwrap_or(0);
        let Some(scope) = self.local_stack.get_mut(index) else {
            return;
        };
        for name in binding_names(pattern) {
            scope.insert(name);
        }
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

    fn add_predeclared_call_binding_names(&mut self, pattern: &BindingPattern<'_>) {
        let Some(scope) = self.call_predeclared_stack.last_mut() else {
            return;
        };
        scope.extend(binding_names(pattern));
    }

    fn add_predeclared_call_binding_name(&mut self, name: &str) {
        let Some(scope) = self.call_predeclared_stack.last_mut() else {
            return;
        };
        scope.insert(name.to_string());
    }

    fn add_predeclared_function_call_binding_name(&mut self, name: &str) {
        let index = self.function_scope_stack.last().copied().unwrap_or(0);
        if let Some(scope) = self.call_predeclared_stack.get_mut(index) {
            scope.insert(name.to_string());
        }
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
        self.local_stack.iter().enumerate().rev().any(|(index, scope)| {
            scope.contains(name)
                || self
                    .call_predeclared_stack
                    .get(index)
                    .is_some_and(|predeclared| predeclared.contains(name))
        })
    }

    fn legacy_local_binding_shadows(&self, name: &str) -> bool {
        let Some(function_scope) = self.function_scope_stack.last().copied() else {
            // Legacy import reachability never modeled lexical bindings in
            // top-level blocks. Keep that conservative behavior even though
            // call reachability needs those scopes for binding correctness.
            return false;
        };
        self.local_stack[function_scope..]
            .iter()
            .rev()
            .any(|scope| scope.contains(name))
    }

    /// Resolve a locally declared callable without walking through a nearer
    /// lexical binding. `Some(None)` means a parameter/value binding hides an
    /// outer function with the same name.
    fn visible_local_function_scope(&self, callee: &str) -> Option<Option<String>> {
        let binding = callee
            .split_once('.')
            .map_or(callee, |(binding, _)| binding);
        let callable = callee.replace('.', "/");
        for (index, locals) in self.local_stack.iter().enumerate().rev() {
            let declared = locals.contains(binding)
                || self
                    .call_predeclared_stack
                    .get(index)
                    .is_some_and(|predeclared| predeclared.contains(binding));
            if !declared {
                continue;
            }
            if index == 0 {
                return Some(
                    self.callable_scopes
                        .contains(&callable)
                        .then(|| callable.clone()),
                );
            }
            if let Some(function_index) = self
                .function_scope_stack
                .iter()
                .rposition(|scope_index| *scope_index <= index)
            {
                let scope = &self.function_stack[function_index];
                let candidate = format!("{scope}/{callable}");
                return Some(
                    self.callable_scopes
                        .contains(&candidate)
                        .then_some(candidate),
                );
            }
            if self.program_scope_active {
                return Some(
                    self.callable_scopes
                        .contains(&callable)
                        .then(|| callable.clone()),
                );
            }
            return Some(None);
        }
        self.callable_scopes
            .contains(&callable)
            .then_some(Some(callable))
    }

    fn callee_shadows_import(&self, callee: &str) -> bool {
        let binding = callee.split_once('.').map_or(callee, |(binding, _)| binding);
        self.legacy_local_binding_shadows(binding)
    }

    fn legacy_has_local_function_scope(&self, callee: &str) -> bool {
        let binding = callee.split_once('.').map_or(callee, |(binding, _)| binding);
        let Some(caller) = self.current_function() else {
            return self.known_function_scopes.contains(binding);
        };
        let mut scope = caller.as_str();
        loop {
            let candidate = format!("{scope}/{binding}");
            if self.known_function_scopes.contains(&candidate) {
                return true;
            }
            let Some((parent, _)) = scope.rsplit_once('/') else {
                return self.known_function_scopes.contains(binding);
            };
            scope = parent;
        }
    }
}
