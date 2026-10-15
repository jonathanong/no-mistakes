use super::*;
use phase_catalog::PhaseCatalog;
use phase_walk::{PhaseWalk, PhaseWrite};
use std::sync::Arc;

pub(in super::super) struct PreparedPhases {
    catalog: Arc<PhaseCatalog>,
    writes: Vec<PhaseWrite>,
    previous: Option<Option<CallableId>>,
    previous_identity: Option<Box<active_reference_phases::ActiveReferencePhases>>,
}
impl ImportCollector {
    pub(in super::super) fn begin_reference_destructuring_phases(
        &mut self,
        target: &AssignmentTarget<'_>,
        rhs: &Expression<'_>,
    ) -> Option<PreparedPhases> {
        if !self.prior_member_mutations.has_reference_proof_demand() {
            return None;
        }
        self.begin_destructuring_phases(target, rhs)
    }
    pub(in super::super) fn begin_destructuring_phases(
        &mut self,
        target: &AssignmentTarget<'_>,
        rhs: &Expression<'_>,
    ) -> Option<PreparedPhases> {
        let owner = self.current_function_id();
        if !self.collect_value_flow
            || self.destructuring_phase_owner == Some(owner)
            || !matches!(
                target,
                AssignmentTarget::ArrayAssignmentTarget(_)
                    | AssignmentTarget::ObjectAssignmentTarget(_)
            )
            || self.prior_member_mutations.is_empty()
            || (!self.has_early_assignment_target(target)
                && !self.prior_member_mutations.has_reference_proof_demand())
        {
            return None;
        }
        let mut walk = PhaseWalk {
            collector: self
                .prior_member_mutations
                .has_reference_proof_demand()
                .then_some(&*self),
            phase: 0,
            cursor: 0,
            segments: Vec::new(),
            writes: Vec::new(),
            nested_pattern: false,
            pattern_depth: 0,
            collect_reference_effects: self.prior_member_mutations.has_reference_proof_demand(),
            effects: None,
        };
        walk.target(target);
        walk.segments.sort_unstable_by_key(|segment| segment.0);
        debug_assert!(walk.segments.windows(2).all(|pair| pair[0].1 <= pair[1].0));
        let PhaseWalk {
            segments,
            writes,
            nested_pattern,
            effects,
            ..
        } = walk;
        let catalog = Arc::new(PhaseCatalog {
            start: target.span().start,
            end: target.span().end,
            segments,
        });
        #[cfg(feature = "test-instrumentation")]
        {
            self.value_flow.early_assignment_catalogs += 1;
            self.value_flow.early_assignment_segments += catalog.segments.len();
        }
        let identity = self
            .prior_member_mutations
            .has_reference_proof_demand()
            .then(|| {
                let inert = !nested_pattern
                    && inert_receiver_rhs::inert_rhs(rhs, &mut self.fresh_receiver_effects);
                Box::new(active_reference_phases::ActiveReferencePhases::new(
                    &catalog,
                    rhs.span().end,
                    &writes,
                    self.destructuring_phase_receipts
                        .as_ref()
                        .map(|receipts| &receipts.scopes),
                    &self.prior_member_mutations,
                    owner,
                    inert,
                    effects,
                ))
            });
        let previous_identity =
            std::mem::replace(&mut self.fresh_receiver_effects.phases, identity);
        let previous = self
            .destructuring_phase_owner
            .replace(self.current_function_id());
        Some(PreparedPhases {
            catalog,
            writes,
            previous,
            previous_identity,
        })
    }
    pub(in super::super) fn end_destructuring_lhs(&mut self, phases: &Option<PreparedPhases>) {
        if let Some(phases) = phases {
            self.destructuring_phase_owner = phases.previous;
            if let Some(receipts) = &mut self.destructuring_phase_receipts {
                receipts.active = false;
            }
        }
    }
    pub(in super::super) fn finish_destructuring_phases(&mut self, phases: Option<PreparedPhases>) {
        if let Some(phases) = phases {
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
