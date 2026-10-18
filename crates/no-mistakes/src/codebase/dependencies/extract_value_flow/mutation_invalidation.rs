use super::*;

impl ImportCollector {
    pub(super) fn invalidate_mutation_bindings(
        &mut self,
        affected: &mutation_projection::Affected,
        context: Option<CallableId>,
        offset: u32,
        member: bool,
    ) {
        self.invalidate_mutation_bindings_indexed(affected, context, offset, member, None);
    }

    pub(super) fn invalidate_mutation_bindings_indexed(
        &mut self,
        affected: &mutation_projection::Affected,
        context: Option<CallableId>,
        offset: u32,
        member: bool,
        aliases: Option<&mutation_alias_index::MutationAliasIndex>,
    ) {
        #[cfg(feature = "test-instrumentation")]
        affected.record_counts(&mut self.value_flow);
        for (scope, names) in affected.iter() {
            for name in names.iter() {
                for index in self
                    .value_binding_indexes
                    .get(scope)
                    .and_then(|names| names.get(name.as_str()))
                    .into_iter()
                    .flatten()
                {
                    let binding = &mut self.value_flow.bindings[*index];
                    binding.mutation_contexts.insert(context);
                    let _stored = mark_invalidated(&mut binding.invalidated_at, offset);
                    #[cfg(feature = "test-instrumentation")]
                    {
                        self.value_flow.reference_invalidation_stores += usize::from(_stored);
                    }
                }
                for index in self
                    .value_parameter_indexes
                    .get(scope)
                    .and_then(|names| names.get(name.as_str()))
                    .into_iter()
                    .flatten()
                {
                    let parameter = &mut self.value_flow.parameters[*index];
                    parameter.mutation_contexts.insert(context);
                    let _stored = mark_invalidated(&mut parameter.invalidated_at, offset);
                    #[cfg(feature = "test-instrumentation")]
                    {
                        self.value_flow.reference_invalidation_stores += usize::from(_stored);
                    }
                }
            }
        }
        if !member {
            return;
        }
        let mut invalidated = Vec::new();
        if let Some(aliases) = aliases {
            for (scope, names) in affected.iter() {
                for name in names.iter() {
                    for slot in aliases.matching(*scope, name) {
                        if let Some(alias) = self.invalidate_callable_alias(*slot, affected, offset)
                        {
                            invalidated.push(alias);
                        }
                    }
                }
            }
        } else {
            for slot in 0..self.callable_aliases.len() {
                if let Some(alias) = self.invalidate_callable_alias(slot, affected, offset) {
                    invalidated.push(alias);
                }
            }
        }
        for (scope, local) in invalidated {
            self.insert_reassigned_callable_at(scope, local);
        }
    }

    fn invalidate_callable_alias(
        &mut self,
        slot: usize,
        affected: &mutation_projection::Affected,
        offset: u32,
    ) -> Option<(usize, String)> {
        #[cfg(feature = "test-instrumentation")]
        {
            self.value_flow.reference_callable_alias_visits += 1;
        }
        let alias = &mut self.callable_aliases[slot].alias;
        let head = alias
            .local
            .split_once('.')
            .map_or(alias.local.as_str(), |(head, _)| head);
        if !affected
            .get(&alias.binding_scope)
            .is_some_and(|names| names.contains(head))
        {
            return None;
        }
        let _stored = mark_invalidated(&mut alias.invalidated_at, offset);
        #[cfg(feature = "test-instrumentation")]
        {
            self.value_flow.reference_invalidation_stores += usize::from(_stored);
        }
        Some((alias.binding_scope, alias.local.clone()))
    }
}

fn mark_invalidated(previous: &mut Option<u32>, offset: u32) -> bool {
    // Deferred projections can arrive after a physically later source write.
    if previous.is_some_and(|previous| previous <= offset) {
        return false;
    }
    *previous = Some(offset);
    true
}

#[cfg(test)]
#[path = "mutation_invalidation_tests.rs"]
mod tests;
