impl ImportCollector {
    fn materialize_deferred_simple_aliases(&mut self) {
        let mut pending = std::mem::take(&mut self.deferred_simple_aliases);
        while !pending.is_empty() {
            let mut remaining = Vec::new();
            let mut progressed = false;
            for candidate in pending {
                if self.has_callable_alias_at(candidate.binding_scope, &candidate.local) {
                    continue;
                }
                if !self
                    .deferred_alias_target_is_callable(&candidate.target, candidate.binding_scope)
                {
                    remaining.push(candidate);
                    continue;
                }
                self.insert_callable_alias(CallableAliasBinding {
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
            if self.has_lexical_binding_at(scope_id, target) {
                if self.has_reassigned_callable_at(scope_id, target) {
                    return false;
                }
                if self.indexed_callable_alias(scope_id, target).is_some() {
                    return true;
                }
                let Some(id) = self.callable_binding_at(scope_id, target) else {
                    return false;
                };
                return !self.callable_scope_ids.iter().any(|(candidate, scope)| {
                    *candidate == id && self.class_scopes.contains(scope)
                });
            }
            scope = self.lexical_scope_parents.get(&scope_id).copied().flatten();
        }
        false
    }
}
