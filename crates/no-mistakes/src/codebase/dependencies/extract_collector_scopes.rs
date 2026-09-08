impl ImportCollector {
    fn record_exported_resource_root(&mut self, name: &str) {
        if self.collect_resource_roots {
            self.exported_resource_roots.insert(name.to_string());
        }
    }

    fn record_exported_resource_scope(&mut self, scope: String) {
        if self.collect_resource_roots {
            self.exported_resource_scopes.insert(scope);
        }
    }

    fn push(&mut self, specifier: &str, kind: ImportKind, byte_offset: usize) {
        self.push_with_side_effect(specifier, kind, byte_offset, false, false);
    }

    fn push_reexport(&mut self, specifier: &str, kind: ImportKind, byte_offset: usize) {
        self.push_with_side_effect(specifier, kind, byte_offset, false, true);
    }

    fn is_exported_top_level_name(&self, name: &str) -> bool {
        self.export_depth > 0 || self.exported_functions.contains(name)
    }

    fn is_exported_top_level_type_name(&self, name: &str) -> bool {
        self.export_depth > 0 || self.later_exported_type_names.contains(name)
    }

    fn push_with_side_effect(
        &mut self,
        specifier: &str,
        kind: ImportKind,
        byte_offset: usize,
        side_effect_only: bool,
        re_export: bool,
    ) {
        let runtime_import = matches!(
            kind,
            ImportKind::Dynamic | ImportKind::Require | ImportKind::RequireResolve
        );
        if self.suppress_imports && !(self.collect_suppressed_runtime_imports && runtime_import) {
            return;
        }
        if !specifier.is_empty() {
            let runtime_reachable = runtime_import
                && self
                    .runtime_reachable_base_depth
                    .is_some_and(|base| self.function_stack.len() <= base + 1);
            self.imports.push(ExtractedImport {
                specifier: specifier.to_string(),
                kind,
                line: import_line_at(&self.line_starts, byte_offset),
                function_scope: self.function_stack.last().cloned(),
                side_effect_only,
                re_export,
                runtime_reachable,
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
            self.push_binding_scope();
        }
    }

    fn push_program_scope(&mut self) {
        self.program_scope_active = true;
        self.push_binding_scope();
    }

    fn pop_program_scope(&mut self) {
        self.pop_binding_scope();
        self.program_scope_active = false;
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
                static_cwd: None,
            });
        }
        self.function_stack.push(scope);
        self.function_scope_stack.push(self.local_stack.len());
        self.push_binding_scope();
    }

    fn pop_function_scope(&mut self, pushed: bool) {
        if pushed {
            self.function_stack.pop();
            self.function_scope_stack.pop();
            self.pop_binding_scope();
        }
    }

    fn push_lexical_scope(&mut self) -> bool {
        self.push_binding_scope();
        true
    }

    fn pop_lexical_scope(&mut self, pushed: bool) {
        if pushed {
            self.pop_binding_scope();
        }
    }

    fn push_binding_scope(&mut self) {
        self.local_stack.push(HashSet::new());
        self.call_predeclared_stack.push(HashSet::new());
        self.call_alias_stack.push(Default::default());
        self.require_factory_bindings.push(Default::default());
        self.type_local_stack.push(HashSet::new());
        self.type_parameter_stack.push(HashSet::new());
    }

    fn pop_binding_scope(&mut self) {
        self.local_stack.pop();
        self.call_predeclared_stack.pop();
        self.call_alias_stack.pop();
        self.require_factory_bindings.pop();
        self.type_local_stack.pop();
        self.type_parameter_stack.pop();
    }
}
