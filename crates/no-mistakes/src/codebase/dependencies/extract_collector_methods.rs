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
        self.known_function_scopes.insert(scope.clone());
        self.callable_scopes.insert(scope.clone());
        // Module callbacks need the same synthetic edge as nested callbacks:
        // a top-level IIFE/callback is reachable from the module root, while a
        // later graph consumer still distinguishes the synthetic invocation.
        self.function_calls.push(FunctionCall {
            caller: self.function_stack.last().cloned(),
            callee: scope.clone(),
            line: 0,
            offset: 0,
            is_callback: true,
            invocation: InvocationKind::Callback,
            target_identity: CallTargetIdentity::RepositoryFunction,
            static_arg: None,
            static_cwd: None,
        });
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
        if !self.local_stack.is_empty() {
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

fn visit_call_expression_with_imports(collector: &mut ImportCollector, call: &CallExpression<'_>) {
    let require_callee = is_require_resolve_callee(&call.callee)
        .then_some("require.resolve")
        .or_else(|| is_require_callee(&call.callee).then_some("require"));
    if let Some(callee) = require_callee {
        if collector.should_record_call(callee) {
            collector.function_calls.push(FunctionCall {
                caller: collector.current_function(),
                callee: callee.to_string(),
                line: import_line_at(&collector.line_starts, call.span.start as usize),
                offset: call.span.start,
                is_callback: false,
                invocation: InvocationKind::Call,
                target_identity: collector.call_target_identity(callee),
                static_arg: call.arguments.first().and_then(static_path_argument),
                static_cwd: None,
            });
        }
    }
    if is_require_resolve_callee(&call.callee) && !collector.local_binding_shadows("require") {
        if let Some(first) = call.arguments.first() {
            if let Some(specifier) = string_literal_argument(first) {
                collector.push(
                    specifier,
                    ImportKind::RequireResolve,
                    call.span.start as usize,
                );
            }
        }
    } else if is_require_callee(&call.callee) && !collector.local_binding_shadows("require") {
        if let Some(first) = call.arguments.first() {
            if let Some(specifier) = string_literal_argument(first) {
                collector.push(specifier, ImportKind::Require, call.span.start as usize);
            }
        }
    } else if let Some(callee) = simple_callee_name(&call.callee) {
        if collector.should_record_call(&callee) {
            let target_identity = collector.call_target_identity(&callee);
            collector.function_calls.push(FunctionCall {
                caller: collector.current_function(),
                static_cwd: static_process_cwd_arg(&callee, &call.arguments),
                callee,
                line: import_line_at(&collector.line_starts, call.span.start as usize),
                offset: call.span.start,
                is_callback: false,
                invocation: InvocationKind::Call,
                target_identity,
                static_arg: call.arguments.first().and_then(static_path_argument),
            });
        }
    } else {
        collector.record_unknown_call(
            import_line_at(&collector.line_starts, call.span.start as usize),
            call.span.start,
            InvocationKind::Call,
        );
    }
}

fn visit_new_expression_with_imports(collector: &mut ImportCollector, new: &NewExpression<'_>) {
    if let Some(callee) = simple_callee_name(&new.callee) {
        if collector.should_record_call(&callee) {
            let target_identity = collector.call_target_identity(&callee);
            collector.function_calls.push(FunctionCall {
                caller: collector.current_function(),
                static_cwd: None,
                callee,
                line: import_line_at(&collector.line_starts, new.span.start as usize),
                offset: new.span.start,
                is_callback: false,
                invocation: InvocationKind::Construct,
                target_identity,
                static_arg: new.arguments.first().and_then(static_path_argument),
            });
        }
    } else {
        collector.record_unknown_call(
            import_line_at(&collector.line_starts, new.span.start as usize),
            new.span.start,
            InvocationKind::Construct,
        );
    }
}

impl ImportCollector {
    fn record_unknown_call(&mut self, line: u32, offset: u32, invocation: InvocationKind) {
        let caller = self.current_function();
        if caller.is_none() {
            self.has_unknown_top_level_call = true;
        }
        self.unknown_callers.push(caller.clone());
        self.unknown_calls.push(UnknownCall {
            caller,
            line,
            offset,
            invocation,
        });
    }
}
