use super::*;

impl ImportCollector {
    pub(in super::super::super) fn end_destructuring_lhs(
        &mut self,
        phases: &Option<PreparedPhases>,
    ) {
        if let Some(phases) = phases {
            self.destructuring_phase_owner = phases.previous;
            if let Some(receipts) = &mut self.destructuring_phase_receipts {
                receipts.active = false;
            }
        }
    }
    pub(in super::super::super) fn finish_destructuring_phases(
        &mut self,
        phases: Option<PreparedPhases>,
    ) {
        if let Some(phases) = phases {
            self.receiver_root_phase_context = phases.previous_root_context;
            if let Some(origins) = &mut self.receiver_only_bindings {
                for (scope, name) in &phases.root_targets {
                    origins.finish_pattern_root(
                        (*scope, name),
                        phases.catalog.start,
                        phases.root_end,
                        phases.owner,
                        phases.root_guaranteed,
                    );
                }
            }
            #[cfg(feature = "test-instrumentation")]
            if let Some(identity) = &self.fresh_receiver_effects.phases {
                identity.record_counts(&mut self.value_flow);
            }
            self.fresh_receiver_effects.phases = phases.previous_identity;
            for write in phases.writes {
                let Some(scope) = self
                    .destructuring_phase_receipts
                    .as_ref()
                    .and_then(|receipts| receipts.scopes.get(&write.target))
                    .copied()
                else {
                    continue;
                };
                self.record_early_assignment(
                    scope,
                    &write.name,
                    &write.path,
                    &phases.catalog,
                    write.phase,
                );
            }
        }
    }
}
