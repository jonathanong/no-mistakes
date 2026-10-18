use super::*;

/// The alias catalog is complete before deferred writer replay. Slot identities
/// remain stable while invalidation updates cutoffs and reassignment sets.
#[derive(Default)]
pub(super) struct MutationAliasIndex {
    heads: FxHashMap<usize, FxHashMap<String, Vec<usize>>>,
}

impl MutationAliasIndex {
    pub(super) fn matching(&self, scope: usize, head: &str) -> &[usize] {
        self.heads
            .get(&scope)
            .and_then(|names| names.get(head))
            .map_or(&[], Vec::as_slice)
    }
}

impl ImportCollector {
    pub(super) fn prepare_mutation_alias_index(&mut self) -> MutationAliasIndex {
        let mut index = MutationAliasIndex::default();
        for (slot, binding) in self.callable_aliases.iter().enumerate() {
            let alias = &binding.alias;
            let head = alias
                .local
                .split_once('.')
                .map_or(alias.local.as_str(), |(head, _)| head);
            let names = index.heads.entry(alias.binding_scope).or_default();
            if let Some(slots) = names.get_mut(head) {
                slots.push(slot);
            } else {
                names.insert(head.to_owned(), vec![slot]);
            }
        }
        #[cfg(feature = "test-instrumentation")]
        {
            self.value_flow.reference_callable_alias_index_builds +=
                usize::from(!self.callable_aliases.is_empty());
            self.value_flow.reference_callable_alias_index_entries += self.callable_aliases.len();
        }
        index
    }
}

#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "mutation_alias_index_tests.rs"]
mod tests;
