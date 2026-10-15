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
            if phases.root_capture {
                if let Some(mut capture) = self.receiver_root_phases.take() {
                    // The normal AST visitor records LHS before RHS. Restore
                    // execution order before selecting any mutable receiver.
                    capture.events.sort_by_key(|event| event.1);
                    if let Some(origins) = &mut self.receiver_only_bindings {
                        let mut targets = FxHashSet::default();
                        for ((scope, name), phase, value, guaranteed) in capture.events {
                            origins.record_pattern_root(
                                (scope, &name),
                                &capture.catalog,
                                phase,
                                phases.owner,
                                value,
                                guaranteed,
                            );
                            targets.insert((scope, name));
                        }
                        for (scope, name) in targets {
                            origins.finish_pattern_root(
                                (scope, &name),
                                capture.catalog.start,
                                phases.root_end,
                                phases.owner,
                                phases.root_guaranteed,
                            );
                        }
                    }
                }
                self.receiver_root_phases = phases.previous_root_capture;
            }
            if phases.defer_mutable {
                self.finish_deferred_mutable_writes(phases.previous_deferred);
            }
            #[cfg(feature = "test-instrumentation")]
            if let Some(identity) = &self.fresh_receiver_effects.phases {
                identity.borrow().record_counts(&mut self.value_flow);
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
