impl CallableFileIndex {
    fn resolve_class_binding_in_scope_chain(
        &self,
        mut binding_scope: usize,
        callee: &str,
    ) -> Option<ResolvedLocalCallee> {
        loop {
            if let Some(target) = self.resolve_class_binding(Some(binding_scope), callee) {
                return Some(target);
            }
            binding_scope = self
                .lexical_scope_parents
                .get(&binding_scope)
                .copied()
                .flatten()?;
        }
    }

    fn resolve_alias(
        &self,
        caller: Option<&str>,
        binding_scope: Option<usize>,
        callee: &str,
    ) -> Option<ResolvedLocalCallee> {
        let mut binding_scope = binding_scope?;
        let mut visited = std::collections::HashSet::new();
        if callee.contains('.') {
            if let Some(target) = self.aliases.get(&(binding_scope, callee.to_string())) {
                return Some(ResolvedLocalCallee {
                    callee: target.clone(),
                    callable_id: None,
                });
            }
        }
        let (binding, member) = callee
            .split_once('.')
            .map_or((callee, None), |(binding, member)| (binding, Some(member)));
        let mut target = binding.to_string();
        let mut resolved_alias = false;
        loop {
            let mut scope = Some(binding_scope);
            let alias = loop {
                let Some(candidate_scope) = scope else {
                    break None;
                };
                if let Some(alias) = self.aliases.get(&(candidate_scope, target.clone())) {
                    break Some((candidate_scope, alias));
                }
                if !resolved_alias {
                    break None;
                }
                scope = self
                    .lexical_scope_parents
                    .get(&candidate_scope)
                    .copied()
                    .flatten();
            };
            if let Some((alias_scope, alias)) = alias {
                let key = (alias_scope, target.clone());
                if !visited.insert(key) {
                    return None;
                }
                resolved_alias = true;
                target = alias.clone();
                let class_target = member
                    .map_or_else(|| target.clone(), |member| format!("{target}.{member}"));
                if let Some(class_scope) =
                    self.resolve_class_binding_in_scope_chain(alias_scope, &class_target)
                {
                    return Some(class_scope);
                }
                let target_binding = target
                    .split_once('.')
                    .map_or(target.as_str(), |(binding, _)| binding);
                if target.contains('.')
                    || self.imported.contains_key(target_binding)
                    || resolve_local_call_scope(
                        caller,
                        None,
                        &target,
                        &self.known_scopes,
                        &self.class_scopes,
                    )
                    .is_some()
                {
                    return Some(ResolvedLocalCallee {
                        callee: member
                            .map_or_else(|| target.clone(), |member| format!("{target}.{member}")),
                        callable_id: None,
                    });
                }
                binding_scope = alias_scope;
                continue;
            }
            return None;
        }
    }
}
