use super::*;
use phase_catalog::PhaseCatalog;
use phase_walk::{PhaseWalk, PhaseWrite};
use std::sync::Arc;

#[path = "phase_finish.rs"]
mod finish;

pub(in super::super) struct PreparedPhases {
    catalog: Arc<PhaseCatalog>,
    writes: Vec<PhaseWrite>,
    root_targets: Vec<(usize, String)>,
    root_end: u32,
    root_guaranteed: bool,
    owner: Option<CallableId>,
    previous_root_context: Option<(Option<CallableId>, usize)>,
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
            || (self.prior_member_mutations.is_empty() && self.receiver_only_bindings.is_none())
            || (!self.has_early_assignment_target(target)
                && !self.prior_member_mutations.has_reference_proof_demand())
        {
            return None;
        }
        let root_context = (owner, self.current_lexical_scope_id());
        let collect_root_origins = self.receiver_only_bindings.is_some()
            && self.receiver_root_phase_context != Some(root_context);
        let mut walk = PhaseWalk {
            collector: self
                .prior_member_mutations
                .has_reference_proof_demand()
                .then_some(&*self),
            phase: 0,
            cursor: 0,
            segments: Vec::new(),
            writes: Vec::new(),
            root_writes: Vec::new(),
            collect_root_origins,
            #[cfg(feature = "test-instrumentation")]
            root_visits: 0,
            conditional: false,
            nested_pattern: false,
            pattern_depth: 0,
            collect_reference_effects: self.prior_member_mutations.has_reference_proof_demand(),
            proof_events: Default::default(),
        };
        if collect_root_origins {
            // Reuse the same evaluation walk for root RHS history. Existing
            // member-reference receipts still cover only the LHS catalog.
            let reference_effects = walk.collect_reference_effects;
            walk.collect_reference_effects = false;
            walk.expression(rhs);
            walk.writes.clear();
            walk.collect_reference_effects = reference_effects;
        }
        walk.target(target);
        walk.segments.sort_unstable_by_key(|segment| segment.0);
        debug_assert!(walk.segments.windows(2).all(|pair| pair[0].1 <= pair[1].0));
        #[cfg(feature = "test-instrumentation")]
        let root_visits = walk.root_visits;
        let PhaseWalk {
            segments,
            writes,
            root_writes,
            nested_pattern,
            proof_events,
            ..
        } = walk;
        #[cfg(feature = "test-instrumentation")]
        {
            self.value_flow.receiver_only_phase_visits += root_visits;
        }
        let root_catalog = (!root_writes.is_empty()).then(|| {
            Arc::new(PhaseCatalog {
                start: target.span().start,
                end: rhs.span().end.max(target.span().end),
                segments: segments.clone(),
            })
        });
        let catalog = Arc::new(PhaseCatalog {
            start: target.span().start,
            end: target.span().end,
            segments: segments
                .into_iter()
                .filter(|segment| segment.0 < target.span().end)
                .collect(),
        });
        let mut root_targets = Vec::new();
        let mut seen_roots = FxHashSet::default();
        if let (Some(origins), Some(root_catalog)) =
            (&mut self.receiver_only_bindings, root_catalog)
        {
            for write in root_writes {
                if seen_roots.insert((write.scope, write.name.clone())) {
                    root_targets.push((write.scope, write.name.clone()));
                }
                origins.record_pattern_root(
                    (write.scope, &write.name),
                    &root_catalog,
                    write.phase,
                    owner,
                    write.value,
                    write.guaranteed,
                );
            }
        }
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
                    (&catalog, rhs.span().end),
                    &writes,
                    self.destructuring_phase_receipts
                        .as_ref()
                        .map(|receipts| &receipts.scopes),
                    &self.prior_member_mutations,
                    owner,
                    inert,
                    proof_events,
                ))
            });
        let previous_identity =
            std::mem::replace(&mut self.fresh_receiver_effects.phases, identity);
        let previous = self
            .destructuring_phase_owner
            .replace(self.current_function_id());
        let previous_root_context = self.receiver_root_phase_context;
        if collect_root_origins {
            // The enclosing walk already covers nested RHS root events, but
            // another callable or lexical class/block scope needs its own facts.
            self.receiver_root_phase_context = Some(root_context);
        }
        Some(PreparedPhases {
            catalog,
            writes,
            root_targets,
            root_end: rhs.span().end.max(target.span().end),
            root_guaranteed: !self.receiver_write_is_conditional
                && self.repeat_depth == self.call_repeat_base,
            owner,
            previous_root_context,
            previous,
            previous_identity,
        })
    }
}
