use super::*;
use phase_catalog::PhaseCatalog;
use phase_walk::{PhaseWalk, PhaseWrite};
use std::sync::Arc;

#[path = "phase_finish.rs"]
mod finish;

pub(in super::super) struct PreparedPhases {
    catalog: Arc<PhaseCatalog>,
    previous_deferred: Option<Box<mutable_deferred::DeferredMutations>>,
    defer_mutable: bool,
    writes: Vec<PhaseWrite>,
    previous_root_capture: Option<Box<root_phase_capture::RootPhaseCapture>>,
    root_capture: bool,
    root_end: u32,
    root_guaranteed: bool,
    owner: Option<CallableId>,
    previous: Option<Option<CallableId>>,
    previous_identity:
        Option<std::rc::Rc<std::cell::RefCell<active_reference_phases::ActiveReferencePhases>>>,
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
        let collect_root_origins = self.receiver_only_bindings.is_some()
            && !self
                .receiver_root_phases
                .as_ref()
                .is_some_and(|capture| capture.owner == owner);
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
            changed_roots: self
                .receiver_only_bindings
                .as_ref()
                .map(|_| Default::default()),
            collect_root_origins,
            root_capture_rhs: false,
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
            walk.root_capture_rhs = true;
            walk.expression(rhs);
            walk.root_capture_rhs = false;
            #[cfg(feature = "test-instrumentation")]
            root_target_metrics::RHS_DISCARDED_WRITES
                .with(|count| count.set(count.get() + walk.writes.len()));
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
        let root_catalog = collect_root_origins.then(|| {
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
        let mut first_changes = fx_map();
        for point in &root_writes {
            first_changes
                .entry(point.name.clone())
                .and_modify(|phase: &mut u32| *phase = (*phase).min(point.phase))
                .or_insert(point.phase);
        }
        let previous_root_capture = root_catalog.and_then(|catalog| {
            self.receiver_root_phases
                .replace(Box::new(root_phase_capture::RootPhaseCapture {
                    catalog,
                    first_changes,
                    points: root_writes
                        .into_iter()
                        .map(|point| (point.target, point))
                        .collect(),
                    owner,
                    conditional_depth: self.receiver_write_conditional_depth,
                    events: Vec::new(),
                }))
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
                std::rc::Rc::new(std::cell::RefCell::new(
                    active_reference_phases::ActiveReferencePhases::new(
                        (&catalog, rhs.span().end),
                        &writes,
                        self.destructuring_phase_receipts
                            .as_ref()
                            .map(|receipts| &receipts.scopes),
                        &self.prior_member_mutations,
                        owner,
                        inert,
                        proof_events,
                    ),
                ))
            });
        let previous_identity =
            std::mem::replace(&mut self.fresh_receiver_effects.phases, identity);
        let previous = self
            .destructuring_phase_owner
            .replace(self.current_function_id());
        let defer_mutable = self.receiver_only_bindings.is_some();
        let previous_deferred = defer_mutable
            .then(|| self.begin_deferred_mutable_writes())
            .flatten();
        Some(PreparedPhases {
            catalog,
            previous_deferred,
            defer_mutable,
            writes,
            previous_root_capture,
            root_capture: collect_root_origins,
            root_end: rhs.span().end.max(target.span().end),
            root_guaranteed: !self.receiver_write_is_conditional
                && self.repeat_depth == self.call_repeat_base,
            owner,
            previous,
            previous_identity,
        })
    }
}
