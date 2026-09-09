impl ImportCollector {
    fn call_target_identity(&self, callee: &str) -> CallTargetIdentity {
        let binding = callee
            .split_once('.')
            .map_or(callee, |(binding, _)| binding);
        if self.local_binding_shadows(binding) {
            if matches!(binding, "globalThis" | "window" | "self" | "global") {
                return CallTargetIdentity::Unknown;
            }
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
            caller_id: self.current_function_id(),
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
            .var_scope_stack
            .last()
            .copied()
            .or_else(|| (!self.local_stack.is_empty()).then_some(0))
        else {
            return;
        };
        let scope_id = self.lexical_scope_ids[index];
        let Some(scope) = self.local_stack.get_mut(index) else {
            return;
        };
        scope.insert(name.to_string());
        self.lexical_binding_names
            .insert((scope_id, name.to_string()));
    }

    fn add_binding_names(&mut self, pattern: &BindingPattern<'_>) {
        let Some(_) = self.local_stack.last() else {
            return;
        };
        let scope_id = self.current_lexical_scope_id();
        let Some(scope) = self.local_stack.last_mut() else {
            return;
        };
        for name in binding_names(pattern) {
            scope.insert(name.clone());
            self.lexical_binding_names.insert((scope_id, name));
        }
    }

    fn add_binding_name(&mut self, name: &str) {
        let Some(_) = self.local_stack.last() else {
            return;
        };
        let scope_id = self.current_lexical_scope_id();
        let Some(scope) = self.local_stack.last_mut() else {
            return;
        };
        scope.insert(name.to_string());
        self.lexical_binding_names
            .insert((scope_id, name.to_string()));
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

    fn record_callable_binding_id(&mut self, name: &str, id: CallableId) {
        let scope = self.current_lexical_scope_id();
        self.record_callable_binding(name);
        self.callable_bindings.insert((scope, name.to_string()), id);
    }

    fn record_class_member_callable_id(
        &mut self,
        class_id: CallableId,
        member: &str,
        member_id: CallableId,
    ) {
        self.class_member_callable_ids
            .insert((class_id, member.to_string(), member_id));
    }

    fn record_aggregate_callable_member_id(
        &mut self,
        class_id: CallableId,
        member: &str,
        member_id: CallableId,
    ) {
        self.aggregate_callable_member_ids
            .insert((class_id, member.to_string(), member_id));
    }

    fn record_class_local_base(&mut self, class_id: CallableId, base: String) {
        self.class_local_bases.insert(class_id, base);
    }

    fn callee_shadows_import(&self, callee: &str) -> bool {
        let binding = callee
            .split_once('.')
            .map_or(callee, |(binding, _)| binding);
        self.local_binding_shadows(binding)
            && (self.imported_bindings.contains(binding)
                || self.predeclared_imported_bindings.contains(binding))
    }

}

include!("extract_callable_scope_helper.rs");
