impl ImportCollector {
    fn materialize_aggregate_aliases(&mut self) {
        for candidate in std::mem::take(&mut self.aggregate_alias_candidates) {
            let AggregateAliasCandidate {
                binding_scope: alias_scope,
                lexical_scope_depth,
                local,
                target,
                declared_at,
                owner,
                owner_id,
            } = candidate;
            let mut scope = Some(alias_scope);
            let source_id = loop {
                let Some(scope_id) = scope else { break None };
                if let Some(id) = self.callable_binding_at(scope_id, &target) {
                    break Some(id);
                }
                scope = self.lexical_scope_parents.get(&scope_id).copied().flatten();
            };
            let Some(source_id) = source_id else {
                continue;
            };
            self.insert_callable_binding_name_at(alias_scope, local.clone());
            self.insert_callable_binding_at(alias_scope, local.clone(), source_id);
            let source_is_class = self
                .callable_scope_ids
                .iter()
                .any(|(id, scope)| *id == source_id && self.class_scopes.contains(scope));
            let members = self
                .aggregate_callable_member_ids
                .get(&source_id)
                .into_iter()
                .flatten()
                .filter(|(member, member_id)| {
                    !source_is_class
                        || self.has_class_member_id(source_id, member, **member_id)
                        || self.has_static_getter_member(source_id, member)
                        || self.has_static_setter_member(source_id, member)
                })
                .map(|(member, _)| member.clone())
                .chain(self.callable_aliases.iter().filter_map(|alias| {
                    (alias.alias.binding_scope == alias_scope)
                        .then(|| alias.alias.local.strip_prefix(&format!("{target}.")))
                        .flatten()
                        .map(str::to_string)
                }))
                .collect::<FxHashSet<_>>();
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
                self.insert_callable_alias(CallableAliasBinding {
                    alias: CallableAlias {
                        scope: owner.clone(),
                        scope_id: owner_id,
                        local: local_member,
                        target: target_member,
                        binding_scope: alias_scope,
                        declared_at,
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
        let mut visited = fx_set();
        loop {
            if !visited.insert(target.clone()) {
                return CallTargetIdentity::Unknown;
            }
            let mut scope = Some(binding_scope);
            let alias = loop {
                let Some(candidate_scope) = scope else {
                    break None;
                };
                if let Some(alias) = self.indexed_callable_alias(candidate_scope, &target) {
                    break Some(alias.target.clone());
                }
                scope = self
                    .lexical_scope_parents
                    .get(&candidate_scope)
                    .copied()
                    .flatten();
            };
            let Some(alias) = alias else { break };
            target = alias;
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

    fn record_object_member_callable_aliases(
        &mut self,
        local: &str,
        object: &ObjectExpression<'_>,
        declared_at: u32,
    ) {
        let mut members = Vec::new();
        for property in &object.properties {
            match property {
                ObjectPropertyKind::SpreadProperty(spread) => match spread_member_aliases(
                    self,
                    &spread.argument,
                ) {
                    Some(spread_members) => {
                        for (member, _) in &spread_members {
                            members.retain(|(existing, _)| existing != member);
                        }
                        members.extend(spread_members);
                    }
                    None => members.clear(),
                },
                ObjectPropertyKind::ObjectProperty(property) => {
                    let Some(member) =
                        crate::codebase::ts_source::static_property_key_name(&property.key)
                    else {
                        continue;
                    };
                    members.retain(|(existing, _)| existing != member);
                    if let Some(target) = self.callable_alias_target(&property.value) {
                        members.push((member.to_string(), target));
                    }
                }
            }
        }
        for (member, target) in members {
            self.push_callable_alias(format!("{local}.{member}"), target, declared_at);
        }
    }
}
