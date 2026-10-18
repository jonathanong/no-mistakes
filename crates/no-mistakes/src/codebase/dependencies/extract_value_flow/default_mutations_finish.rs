use super::*;

impl ImportCollector {
    pub(in super::super::super) fn finish_default_mutations(&mut self) {
        #[cfg(feature = "test-instrumentation")]
        if let Some(origins) = &self.receiver_only_bindings {
            origins.record_counts(&mut self.value_flow);
        }
        #[cfg(feature = "test-instrumentation")]
        self.fresh_receiver_effects
            .record_counts(&mut self.value_flow);
        let Some(index) = self.parameter_default_origins.take() else {
            return;
        };
        let mut histories = std::mem::take(&mut self.prior_member_mutations);
        let components = histories.finish();
        #[cfg(feature = "test-instrumentation")]
        histories.record_reference_counts(&mut self.value_flow);
        let mut retained = fx_map();
        for (component, mut history) in components {
            history.finish_early_assignments();
            #[cfg(feature = "test-instrumentation")]
            {
                self.value_flow.early_assignment_normalization_visits +=
                    history.early_phase_entries();
            }
            let history = std::sync::Arc::new(history);
            for (scope, name) in component {
                retained
                    .entry(scope)
                    .or_insert_with(fx_map)
                    .insert(name.clone(), std::sync::Arc::clone(&history));
                // A late alias captures the already mutated receiver, including
                // a synthetic default export. Apply the shared cutoff once.
                if let Some(summary) = &history.summary {
                    for slot in self
                        .value_binding_indexes
                        .get(&scope)
                        .and_then(|names| names.get(name.as_str()))
                        .into_iter()
                        .flatten()
                    {
                        let binding = &mut self.value_flow.bindings[*slot];
                        binding.invalidated_at = Some(
                            binding
                                .invalidated_at
                                .map_or(summary.offset, |offset| offset.min(summary.offset)),
                        );
                        binding
                            .mutation_contexts
                            .extend(summary.contexts.iter().copied());
                    }
                }
                for receiver in index
                    .get(&scope)
                    .and_then(|names| names.get(name.as_str()))
                    .into_iter()
                    .flatten()
                {
                    let parameter = &mut self.value_flow.parameters[receiver.parameter];
                    let mutations = std::sync::Arc::make_mut(
                        parameter
                            .default_mutations
                            .get_or_insert_with(|| std::sync::Arc::new(fx_map())),
                    );
                    mutations.entry(receiver.origin).or_default().push(
                        ValueDefaultMutationSource {
                            path: receiver.path.clone(),
                            history: std::sync::Arc::clone(&history),
                        },
                    );
                }
            }
        }
        self.value_flow.receiver_mutations = Some(std::sync::Arc::new(retained));
    }
}
