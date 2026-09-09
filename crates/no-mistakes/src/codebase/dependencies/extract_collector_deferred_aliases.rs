impl ImportCollector {
    fn materialize_deferred_simple_aliases(&mut self) {
        let mut pending = std::mem::take(&mut self.deferred_simple_aliases);
        while !pending.is_empty() {
            let mut remaining = Vec::new();
            let mut progressed = false;
            for candidate in pending {
                if self.callable_aliases.iter().any(|alias| {
                    alias.alias.binding_scope == candidate.binding_scope
                        && alias.alias.local == candidate.local
                }) {
                    continue;
                }
                if !self.deferred_alias_target_is_callable(
                    &candidate.target,
                    candidate.binding_scope,
                ) {
                    remaining.push(candidate);
                    continue;
                }
                self.callable_aliases.push(CallableAliasBinding {
                    alias: CallableAlias {
                        scope: candidate.owner,
                        scope_id: candidate.owner_id,
                        local: candidate.local,
                        target: candidate.target,
                        binding_scope: candidate.binding_scope,
                        declared_at: candidate.declared_at,
                        invalidated_at: None,
                    },
                    lexical_scope_depth: candidate.lexical_scope_depth,
                });
                progressed = true;
            }
            if !progressed {
                break;
            }
            pending = remaining;
        }
    }

    fn deferred_alias_target_is_callable(&self, target: &str, binding_scope: usize) -> bool {
        let mut scope = Some(binding_scope);
        while let Some(scope_id) = scope {
            if self
                .callable_bindings
                .contains_key(&(scope_id, target.to_string()))
                || self.callable_aliases.iter().any(|alias| {
                    alias.alias.binding_scope == scope_id && alias.alias.local == target
                })
            {
                return true;
            }
            scope = self.lexical_scope_parents.get(&scope_id).copied().flatten();
        }
        false
    }
}
