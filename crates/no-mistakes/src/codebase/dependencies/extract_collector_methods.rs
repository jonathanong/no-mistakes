impl ImportCollector {
    fn push(&mut self, specifier: &str, kind: ImportKind) {
        self.push_with_side_effect(specifier, kind, false);
    }

    fn push_with_side_effect(&mut self, specifier: &str, kind: ImportKind, side_effect_only: bool) {
        if self.suppress_imports
            && !(self.collect_suppressed_runtime_imports
                && matches!(kind, ImportKind::Dynamic | ImportKind::Require))
        {
            return;
        }
        if !specifier.is_empty() {
            self.imports.push(ExtractedImport {
                specifier: specifier.to_string(),
                kind,
                function_scope: self.function_stack.last().cloned(),
                side_effect_only,
            });
        }
    }

    fn push_function_scope(&mut self, name: Option<String>) {
        if let Some(name) = name {
            let scope = self
                .function_stack
                .last()
                .map(|parent| format!("{parent}/{name}"))
                .unwrap_or(name);
            self.known_function_scopes.insert(scope.clone());
            if self.export_depth > 0 && self.function_stack.is_empty() {
                self.exported_functions.insert(scope.clone());
            }
            self.function_stack.push(scope);
            self.function_scope_stack.push(self.local_stack.len());
            self.local_stack.push(HashSet::new());
            self.type_local_stack.push(HashSet::new());
            self.type_parameter_stack.push(HashSet::new());
        }
    }

    fn push_anonymous_function_scope(&mut self) {
        self.anonymous_scope_count += 1;
        let name = format!("<anonymous:{}>", self.anonymous_scope_count);
        let scope = self
            .function_stack
            .last()
            .map(|parent| format!("{parent}/{name}"))
            .unwrap_or(name);
        if let Some(parent) = self.function_stack.last() {
            self.function_calls.push(FunctionCall {
                caller: Some(parent.clone()),
                callee: scope.clone(),
                static_arg: None,
            });
        }
        self.function_stack.push(scope);
        self.function_scope_stack.push(self.local_stack.len());
        self.local_stack.push(HashSet::new());
        self.type_local_stack.push(HashSet::new());
        self.type_parameter_stack.push(HashSet::new());
    }

    fn pop_function_scope(&mut self, pushed: bool) {
        if pushed {
            self.function_stack.pop();
            self.function_scope_stack.pop();
            self.local_stack.pop();
            self.type_local_stack.pop();
            self.type_parameter_stack.pop();
        }
    }

    fn push_lexical_scope(&mut self) -> bool {
        if self.current_function().is_some() {
            self.local_stack.push(HashSet::new());
            self.type_local_stack.push(HashSet::new());
            self.type_parameter_stack.push(HashSet::new());
            true
        } else {
            false
        }
    }

    fn pop_lexical_scope(&mut self, pushed: bool) {
        if pushed {
            self.local_stack.pop();
            self.type_local_stack.pop();
            self.type_parameter_stack.pop();
        }
    }
}
