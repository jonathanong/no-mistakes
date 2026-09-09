impl CallableFileIndex {
    fn resolve_class_binding(
        &self,
        binding_scope: Option<usize>,
        callee: &str,
    ) -> Option<ResolvedLocalCallee> {
        let binding_scope = binding_scope?;
        let (binding, member) = callee
            .split_once('.')
            .map_or((callee, None), |(binding, member)| (binding, Some(member)));
        let scope = self
            .class_bindings
            .get(&(binding_scope, binding.to_string()))?;
        let (target_scope, callable_id) = match member {
            Some(member) => {
                let (owner, member_id) =
                    self.resolve_static_member(binding_scope, scope, member)?;
                (owner.scope.as_str(), member_id)
            }
            None => (scope.scope.as_str(), scope.class_id),
        };
        Some(ResolvedLocalCallee {
            callee: member.map_or_else(
                || target_scope.to_string(),
                |member| format!("{target_scope}.{member}"),
            ),
            callable_id: Some(callable_id),
        })
    }

    fn resolve_static_member<'a>(
        &'a self,
        binding_scope: usize,
        scope: &'a ClassBindingTarget,
        member: &str,
    ) -> Option<(
        &'a ClassBindingTarget,
        crate::codebase::dependencies::extract::CallableId,
    )> {
        let mut scope = scope;
        let mut visited = std::collections::HashSet::new();
        loop {
            if let Some(member_id) = scope.static_member_ids.get(member) {
                return Some((scope, *member_id));
            }
            let base = scope.local_base.as_ref()?;
            let mut lexical_scope = Some(binding_scope);
            let base_scope = loop {
                let candidate_scope = lexical_scope?;
                if let Some(base_scope) = self.class_bindings.get(&(candidate_scope, base.clone())) {
                    break base_scope;
                }
                lexical_scope = self
                    .lexical_scope_parents
                    .get(&candidate_scope)
                    .copied()
                    .flatten();
            };
            if !visited.insert(base_scope.class_id) {
                return None;
            }
            scope = base_scope;
        }
    }
}
