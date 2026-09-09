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
        let callable_alias = self
            .callable_alias_index
            .contains_key(&(binding_scope, binding.to_string()));
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
            let is_class = self
                .callable_scope_ids
                .iter()
                .any(|(id, scope)| id == class_id && self.class_scopes.contains(scope));
            if is_class {
                return self.has_class_static_member(*class_id, binding_scope, member);
            }
            return self.function_calls.iter().any(|call| {
                call.invocation == InvocationKind::Membership
                    && call.caller_id == Some(*class_id)
                    && call.callee == member
            });
        }
        if callable_alias {
            return true;
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

impl ImportCollector {
    fn has_class_static_member(
        &self,
        mut class_id: CallableId,
        binding_scope: usize,
        member: &str,
    ) -> bool {
        let mut seen = HashSet::new();
        loop {
            if !seen.insert(class_id) {
                return false;
            }
            if self.class_member_callable_ids.iter().any(
                |(candidate_class_id, candidate_member, _)| {
                    *candidate_class_id == class_id && candidate_member == member
                },
            ) {
                return true;
            }
            let Some(base) = self.class_local_bases.get(&class_id) else {
                return false;
            };
            let mut scope = Some(binding_scope);
            let base_id = loop {
                let Some(scope_id) = scope else { break None };
                if let Some(id) = self.callable_bindings.get(&(scope_id, base.clone())) {
                    break Some(*id);
                }
                scope = self.lexical_scope_parents.get(&scope_id).copied().flatten();
            };
            let Some(base_id) = base_id else {
                return false;
            };
            let base_is_class = self
                .callable_scope_ids
                .iter()
                .any(|(id, scope)| *id == base_id && self.class_scopes.contains(scope));
            if !base_is_class {
                return false;
            }
            class_id = base_id;
        }
    }
}
