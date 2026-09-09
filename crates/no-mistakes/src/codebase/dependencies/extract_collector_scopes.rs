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
            // Flag a runtime import in the callback directly forming an exported
            // value (e.g. `next/dynamic(() => import('./Foo'))`) so the edge
            // survives reachability analysis. Limit it to one function level below
            // the exported initializer so deeper, uninvoked nested imports keep
            // their normal call-scope pruning.
            let runtime_reachable = runtime_import
                && self
                    .runtime_reachable_base_depth
                    .is_some_and(|base| self.function_stack.len() <= base + 1);
            self.imports.push(ExtractedImport {
                specifier: specifier.to_string(),
                kind,
                line: import_line_at(&self.line_starts, byte_offset),
                function_scope: self.function_stack.last().cloned(),
                function_scope_id: self.current_function_id(),
                side_effect_only,
                re_export,
                runtime_reachable,
            });
        }
    }

    fn push_function_scope(&mut self, name: Option<String>, id: CallableId) {
        self.push_function_scope_for_binding(name, None, id);
    }

    fn push_function_scope_for_binding(
        &mut self,
        name: Option<String>,
        binding_scope: Option<usize>,
        id: CallableId,
    ) {
        if let Some(name) = name {
            let scope = self.callable_scope_name_for_binding(&name, binding_scope);
            self.known_function_scopes.insert(scope.clone());
            self.callable_scope_ids.insert((id, scope.clone()));
            if self.export_depth > 0 && self.function_stack.is_empty() {
                self.exported_functions.insert(scope.clone());
            }
            self.function_stack.push(scope);
            self.function_id_stack.push(id);
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

    fn callable_scope_name(&self, name: &str) -> String {
        self.callable_scope_name_for_binding(name, None)
    }

    fn callable_scope_name_for_binding(&self, name: &str, _binding_scope: Option<usize>) -> String {
        let parent = self.function_stack.last();
        // Lexical identity is carried by `CallableId`; public callable names
        // stay source-level and must never expose collector implementation
        // counters such as `<scope:N>`.
        let name = name.to_string();
        parent
            .map(|parent| format!("{parent}/{name}"))
            .unwrap_or(name)
    }

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
            offset: 0,
            is_callback: true,
            invocation: InvocationKind::Callback,
            target_identity: CallTargetIdentity::RepositoryFunction,
            callee_binding_scope: None,
            static_arg: None,
            static_cwd: None,
        });
        self.function_stack.push(scope);
        self.function_id_stack.push(id);
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

    fn pop_function_scope(&mut self, pushed: bool) {
        if pushed {
            self.function_stack.pop();
            self.function_id_stack.pop();
            self.var_scope_stack.pop();
            self.local_stack.pop();
            self.lexical_scope_ids.pop();
            self.type_local_stack.pop();
            self.type_parameter_stack.pop();
        }
    }

    fn current_syntactic_caller(&self) -> Option<String> {
        self.syntactic_caller_stack.last().cloned()
    }

    fn current_function_id(&self) -> Option<CallableId> {
        self.function_id_stack.last().copied()
    }

    fn callable_binding_id(&self, name: &str) -> Option<CallableId> {
        self.callable_binding_at(self.current_lexical_scope_id(), name)
    }

    fn push_syntactic_caller(&mut self, name: Option<String>) -> bool {
        if let Some(name) = name {
            self.syntactic_caller_stack.push(name);
            true
        } else {
            false
        }
    }

    fn pop_syntactic_caller(&mut self, pushed: bool) {
        if pushed {
            self.syntactic_caller_stack.pop();
        }
    }

    fn push_lexical_scope(&mut self) -> bool {
        if !self.local_stack.is_empty() {
            self.local_stack.push(fx_set());
            let lexical_scope_id = self.next_lexical_scope_id;
            self.lexical_scope_parents
                .insert(lexical_scope_id, self.lexical_scope_ids.last().copied());
            self.lexical_scope_ids.push(lexical_scope_id);
            self.next_lexical_scope_id += 1;
            self.type_local_stack.push(fx_set());
            self.type_parameter_stack.push(fx_set());
            true
        } else {
            false
        }
    }

    fn pop_lexical_scope(&mut self, pushed: bool) {
        if pushed {
            self.local_stack.pop();
            self.lexical_scope_ids.pop();
            self.type_local_stack.pop();
            self.type_parameter_stack.pop();
        }
    }
}
