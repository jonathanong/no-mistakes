use super::*;
#[path = "affected_receivers.rs"]
mod affected;
pub(super) use affected::Affected;
#[path = "mutation_eligibility.rs"]
mod eligibility;
pub(super) use eligibility::MutationInfo;
#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "mutation_projection_differential_tests.rs"]
mod differential_tests;
#[path = "mutation_pending.rs"]
mod pending_work;
#[path = "mutation_pending_queue.rs"]
mod queue;
pub(super) use queue::{PendingMutation, Queue as PendingQueue};
impl ImportCollector {
    pub(super) fn project_member_mutation<R: nested::RootCoordinate>(
        &mut self,
        pending: PendingQueue<PendingMutation<R>>,
        assigned: Option<(Option<String>, u32)>,
        info: MutationInfo<'_>,
        affected: &mut Affected,
    ) {
        #[cfg(all(test, feature = "test-instrumentation"))]
        if differential_tests::LEGACY.with(std::cell::Cell::get) {
            return self.project_member_mutation_legacy(
                pending.into_vec(),
                assigned,
                info,
                affected,
            );
        }
        #[cfg(all(test, feature = "test-instrumentation"))]
        {
            admission_tests::writer(self.prior_member_mutations.has_assigned_origins());
            if admission_tests::LEGACY.with(std::cell::Cell::get) {
                return self.walk_projected_mutation::<R, true>(pending, assigned, info, affected);
            }
        }
        // Only finish_assignment_rhs publishes origins, after this writer returns.
        // One immutable admission erases RHS-only work for the ordinary traversal.
        if self.projection_rhs_origins() {
            self.walk_projected_mutation::<R, true>(pending, assigned, info, affected);
        } else {
            self.walk_projected_mutation::<R, false>(pending, assigned, info, affected);
        }
    }
    fn projected_rhs_admitted<const RHS: bool>(&self) -> bool {
        #[cfg(all(test, feature = "test-instrumentation"))]
        return admission_tests::admitted::<RHS>(self);
        #[cfg(not(all(test, feature = "test-instrumentation")))]
        RHS
    }
}

include!("mutation_projection_walk.rs");

#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "mutation_projection_admission_tests.rs"]
pub(super) mod admission_tests;
