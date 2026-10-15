use super::*;

impl ImportCollector {
    pub(super) fn invalidate_mutation_bindings(
        &mut self,
        affected: &mutation_projection::Affected,
        metadata: mutation_metadata::MutationMetadata,
        member: bool,
    ) {
        for (scope, names) in affected {
            for name in names {
                for index in self
                    .value_binding_indexes
                    .get(scope)
                    .and_then(|names| names.get(name.as_str()))
                    .into_iter()
                    .flatten()
                {
                    let binding = &mut self.value_flow.bindings[*index];
                    binding.mutation_contexts.insert(metadata.context);
                    mark_invalidated(&mut binding.invalidated_at, metadata.offset);
                }
                for index in self
                    .value_parameter_indexes
                    .get(scope)
                    .and_then(|names| names.get(name.as_str()))
                    .into_iter()
                    .flatten()
                {
                    let parameter = &mut self.value_flow.parameters[*index];
                    parameter.mutation_contexts.insert(metadata.context);
                    mark_invalidated(&mut parameter.invalidated_at, metadata.offset);
                }
            }
        }
        let mut invalidated = Vec::new();
        for binding in &mut self.callable_aliases {
            let alias = &mut binding.alias;
            let head = alias
                .local
                .split_once('.')
                .map_or(alias.local.as_str(), |(head, _)| head);
            if member
                && affected
                    .get(&alias.binding_scope)
                    .is_some_and(|names| names.contains(head))
            {
                mark_invalidated(&mut alias.invalidated_at, metadata.offset);
                invalidated.push((alias.binding_scope, alias.local.clone()));
            }
        }
        for (scope, local) in invalidated {
            self.insert_reassigned_callable_at(scope, local);
        }
    }
}

fn mark_invalidated(previous: &mut Option<u32>, offset: u32) {
    // Deferred projections can arrive after a physically later source write.
    *previous = Some(previous.map_or(offset, |previous| previous.min(offset)));
}
