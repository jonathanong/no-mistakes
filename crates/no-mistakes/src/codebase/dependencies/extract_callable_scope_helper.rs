impl ImportCollector {
    fn has_local_function_scope(&self, callee: &str) -> bool {
        let binding = callee
            .split_once('.')
            .map_or(callee, |(binding, _)| binding);
        let Some(binding_scope) = self.callee_binding_scope(callee) else {
            return false;
        };
        if self
            .reassigned_callable_binding_ids
            .contains(&(binding_scope, binding.to_string()))
            || self
                .reassigned_callable_binding_ids
                .contains(&(binding_scope, callee.to_string()))
        {
            return false;
        }
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
        if let Some((_, member)) = callee.split_once('.') {
            if member == "constructor" {
                return false;
            }
            let Some(class_id) = self
                .callable_bindings
                .get(&(binding_scope, binding.to_string()))
            else {
                return false;
            };
            return self.function_calls.iter().any(|call| {
                call.invocation == InvocationKind::Membership
                    && call.caller_id == Some(*class_id)
                    && call.callee == member
            });
        }
        if let Some(class_id) = self
            .callable_bindings
            .get(&(binding_scope, binding.to_string()))
        {
            if self
                .callable_scope_ids
                .iter()
                .any(|(id, scope)| id == class_id && self.class_scopes.contains(scope))
            {
                return true;
            }
        }
        let Some(caller) = self.current_function() else {
            let member = binding.replace('.', "/");
            return self.callable_scopes.contains(binding)
                || self.known_function_scopes.contains(&member);
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
