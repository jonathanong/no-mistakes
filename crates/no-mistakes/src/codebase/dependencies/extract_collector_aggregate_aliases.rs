impl ImportCollector {
    fn materialize_aggregate_aliases(&mut self) {
        for candidate in std::mem::take(&mut self.aggregate_alias_candidates) {
            let AggregateAliasCandidate {
                binding_scope: alias_scope,
                lexical_scope_depth,
                local,
                target,
                owner,
                owner_id,
            } = candidate;
            let mut scope = Some(alias_scope);
            let source_id = loop {
                let Some(scope_id) = scope else { break None };
                if let Some(id) = self.callable_bindings.get(&(scope_id, target.clone())) {
                    break Some(*id);
                }
                scope = self.lexical_scope_parents.get(&scope_id).copied().flatten();
            };
            let Some(source_id) = source_id else {
                continue;
            };
            self.callable_binding_ids
                .insert((alias_scope, local.clone()));
            self.callable_bindings
                .insert((alias_scope, local.clone()), source_id);
            let source_is_class = self
                .callable_scope_ids
                .iter()
                .any(|(id, scope)| *id == source_id && self.class_scopes.contains(scope));
            let members = self
                .aggregate_callable_member_ids
                .iter()
                .filter(|(aggregate_id, member, member_id)| {
                    *aggregate_id == source_id
                        && (!source_is_class
                            || self.class_member_callable_ids.contains(&(
                                *aggregate_id,
                                member.clone(),
                                *member_id,
                            )))
                })
                .map(|(_, member, _)| member.clone())
                .chain(self.callable_aliases.iter().filter_map(|alias| {
                    (alias.alias.binding_scope == alias_scope)
                        .then(|| alias.alias.local.strip_prefix(&format!("{target}.")))
                        .flatten()
                        .map(str::to_string)
                }))
                .collect::<HashSet<_>>();
            for member in members {
                let local_member = format!("{local}.{member}");
                let target_member = format!("{target}.{member}");
                let target_identity =
                    self.materialized_alias_target_identity(alias_scope, &target_member);
                for call in &mut self.function_calls {
                    if call.callee == local_member && call.callee_binding_scope == Some(alias_scope)
                    {
                        call.target_identity = target_identity;
                    }
                }
                self.callable_aliases.push(CallableAliasBinding {
                    alias: CallableAlias {
                        scope: owner.clone(),
                        scope_id: owner_id,
                        local: local_member,
                        target: target_member,
                        binding_scope: alias_scope,
                        invalidated_at: None,
                    },
                    lexical_scope_depth,
                });
            }
        }
    }

    fn materialized_alias_target_identity(
        &self,
        binding_scope: usize,
        initial_target: &str,
    ) -> CallTargetIdentity {
        let mut target = initial_target.to_string();
        let mut visited = HashSet::new();
        loop {
            if !visited.insert(target.clone()) {
                return CallTargetIdentity::Unknown;
            }
            let mut scope = Some(binding_scope);
            let alias = loop {
                let Some(candidate_scope) = scope else {
                    break None;
                };
                if let Some(alias) = self.callable_aliases.iter().find(|alias| {
                    alias.alias.binding_scope == candidate_scope && alias.alias.local == target
                }) {
                    break Some(&alias.alias.target);
                }
                scope = self
                    .lexical_scope_parents
                    .get(&candidate_scope)
                    .copied()
                    .flatten();
            };
            let Some(alias) = alias else { break };
            target = alias.clone();
        }

        let binding = target
            .split_once('.')
            .map_or(target.as_str(), |(binding, _)| binding);
        if self.imported_bindings.contains(binding)
            || self.predeclared_imported_bindings.contains(binding)
        {
            CallTargetIdentity::ModuleExport
        } else if self.callable_scopes.contains(&target)
            || self.callable_scopes.contains(&target.replace('.', "/"))
            || self.known_function_scopes.contains(&target)
            || self
                .known_function_scopes
                .contains(&target.replace('.', "/"))
        {
            CallTargetIdentity::RepositoryFunction
        } else {
            CallTargetIdentity::Unknown
        }
    }
}
