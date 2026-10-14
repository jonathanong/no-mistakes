use super::*;
use phase_catalog::PhaseCatalog;
use phase_walk::{PhaseWalk, PhaseWrite};
use std::sync::Arc;

pub(in super::super) struct PreparedPhases {
    catalog: Arc<PhaseCatalog>,
    writes: Vec<PhaseWrite>,
    previous: Option<Option<CallableId>>,
}
impl ImportCollector {
    pub(in super::super) fn begin_destructuring_phases(
        &mut self,
        target: &AssignmentTarget<'_>,
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
            || !self.has_early_assignment_target(target)
        {
            return None;
        }
        let mut walk = PhaseWalk {
            phase: 0,
            cursor: 0,
            segments: Vec::new(),
            writes: Vec::new(),
        };
        walk.target(target);
        walk.segments.sort_unstable_by_key(|segment| segment.0);
        debug_assert!(walk.segments.windows(2).all(|pair| pair[0].1 <= pair[1].0));
        let catalog = Arc::new(PhaseCatalog {
            start: target.span().start,
            end: target.span().end,
            segments: walk.segments,
        });
        let writes = walk.writes;
        #[cfg(feature = "test-instrumentation")]
        {
            self.value_flow.early_assignment_catalogs += 1;
            self.value_flow.early_assignment_segments += catalog.segments.len();
        }
        let previous = self
            .destructuring_phase_owner
            .replace(self.current_function_id());
        Some(PreparedPhases {
            catalog,
            writes,
            previous,
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
