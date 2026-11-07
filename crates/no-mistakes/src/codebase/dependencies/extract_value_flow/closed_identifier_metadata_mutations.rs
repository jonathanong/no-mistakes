use super::*;
use crate::fx::FxHashSet;

impl ImportCollector {
    pub(in super::super::super) fn finalize_metadata_sources(&mut self) {
        if !self.collect_value_flow || self.metadata_registry_names.is_empty() {
            return;
        }
        let mutated: FxHashSet<_> = self
            .value_flow
            .registry_mutations
            .iter()
            .map(|mutation| (mutation.scope, mutation.name.as_str()))
            .collect();
        let mut sources: Vec<_> = self
            .value_flow
            .bounded_bindings
            .iter()
            .filter(|binding| {
                binding.requires_guard && mutated.contains(&(binding.scope, binding.name.as_str()))
            })
            .filter_map(|binding| binding.source.as_ref())
            .filter(|(name, scope)| !mutated.contains(&(*scope, name.as_str())))
            .cloned()
            .collect();
        sources.sort_unstable();
        sources.dedup();
        drop(mutated);
        self.value_flow.registry_mutations.extend(
            sources
                .into_iter()
                .map(|(name, scope)| RegistryMutation { name, scope }),
        );
    }
}
