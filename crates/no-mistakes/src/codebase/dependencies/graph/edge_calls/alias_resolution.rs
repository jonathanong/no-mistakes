impl CallableFileIndex {
    fn resolve_local_callable_id(
        &self,
        mut binding_scope: Option<usize>,
        callee: &str,
    ) -> Option<crate::codebase::dependencies::extract::CallableId> {
        let binding = callee.rsplit('/').next()?;
        if binding.contains('.') {
            return None;
        }
        while let Some(scope) = binding_scope {
            if let Some(id) = self.callable_bindings.get(&(scope, binding.to_string())) {
                return Some(*id);
            }
            binding_scope = self.lexical_scope_parents.get(&scope).copied().flatten();
        }
        None
    }

    fn resolve_class_binding_in_scope_chain(
        &self,
        mut binding_scope: usize,
        callee: &str,
        invocation: InvocationKind,
    ) -> Option<ResolvedLocalCallee> {
        loop {
            if let Some(target) = self.resolve_class_binding(Some(binding_scope), callee, invocation) {
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
        offset: u32,
        caller_id: Option<crate::codebase::dependencies::extract::CallableId>,
        invocation: InvocationKind,
    ) -> Option<ResolvedLocalCallee> {
        let call_binding_scope = binding_scope;
        let mut binding_scope = binding_scope?;
        let mut visited = fx_set();
        if callee.contains('.') {
            // Dotted members hop until the chain ends: `calls.run` -> `invoke`
            // -> `target`. Returning after the first alias would disagree with
            // import reachability on the same facts.
            let mut target = callee.to_string();
            let mut resolved_alias = false;
            loop {
                let mut scope = Some(binding_scope);
                let alias = loop {
                    let Some(candidate_scope) = scope else {
                        break None;
                    };
                    if let Some(alias) = self
                        .aliases
                        .get(&(candidate_scope, target.clone()))
                        .filter(|alias| {
                            self.alias_live_at(
                                alias,
                                candidate_scope,
                                offset,
                                call_binding_scope,
                                caller_id,
                            )
                        })
                    {
                        break Some((candidate_scope, alias));
                    }
                    scope = self
                        .lexical_scope_parents
                        .get(&candidate_scope)
                        .copied()
                        .flatten();
                };
                let Some((alias_scope, alias)) = alias else {
                    if resolved_alias {
                        if let Some(class_scope) = self.resolve_class_binding_in_scope_chain(
                            binding_scope,
                            &target,
                            invocation,
                        ) {
                            return Some(class_scope);
                        }
                    }
                    return resolved_alias
                        .then(|| {
                            self.target_binding_live(
                                binding_scope,
                                &target,
                                offset,
                                call_binding_scope,
                                caller_id,
                            )
                            .then_some(ResolvedLocalCallee {
                                callee: target,
                                callable_id: None,
                            })
                        })
                        .flatten();
                };
                if !visited.insert((alias_scope, target)) {
                    return None;
                }
                resolved_alias = true;
                target = alias.target.clone();
                binding_scope = alias_scope;
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
                if let Some(alias) = self
                    .aliases
                    .get(&(candidate_scope, target.clone()))
                    .filter(|alias| {
                        self.alias_live_at(
                            alias,
                            candidate_scope,
                            offset,
                            call_binding_scope,
                            caller_id,
                        )
                    })
                {
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
                target = alias.target.clone();
                let class_target =
                    member.map_or_else(|| target.clone(), |member| format!("{target}.{member}"));
                if let Some(class_scope) =
                    self.resolve_class_binding_in_scope_chain(alias_scope, &class_target, invocation)
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
                    if !self.target_binding_live(
                        alias_scope,
                        target_binding,
                        offset,
                        call_binding_scope,
                        caller_id,
                    ) {
                        return None;
                    }
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
