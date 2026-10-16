use super::*;

impl ImportCollector {
    pub(in super::super) fn first_projected_receiver(
        &mut self,
        key: &(usize, String),
        root_read: bool,
        #[cfg(feature = "test-instrumentation")] binding_visits: &mut usize,
    ) -> Option<usize> {
        self.value_binding_indexes
            .get(&key.0)
            .and_then(|names| names.get(key.1.as_str()))
            .map_or(&[][..], Vec::as_slice)
            .iter()
            .position(|index| {
                #[cfg(feature = "test-instrumentation")]
                {
                    *binding_visits += 1;
                }
                let value = &self.value_flow.bindings[*index].value;
                !matches!(value, ValueExpression::Named { name, .. } if !name.contains('.'))
                    || (root_read && self.references_mutable_receiver(value))
            })
    }
}

impl ImportCollector {
    pub(in super::super) fn projected_root_version<R: nested::RootCoordinate>(
        &mut self,
        key: &(usize, String),
        root_read: R,
    ) -> Option<mutable_origins::RootVersion> {
        root_read.read().and_then(|read| {
            #[cfg(feature = "test-instrumentation")]
            {
                self.value_flow.reference_root_probes += 1;
            }
            self.mutable_receiver_version(key, read)
        })
    }
}

impl ImportCollector {
    pub(in super::super) fn finish_projected_receiver<R: nested::RootCoordinate>(
        &mut self,
        traversal: &mut receiver_traversal::ReceiverTraversal,
        key: &(usize, String),
        root_read: R,
        assigned: bool,
    ) {
        // RHS exits own their separate receipt-qualified active stack.
        if assigned {
            traversal.exit_assigned(key);
            return;
        }
        if root_read.read().is_none() {
            traversal.exit_ordinary(key);
            return;
        }
        if root_read.read().is_some() && traversal.exit_single_version(key) {
            return;
        }
        let version = self.projected_root_version(key, root_read);
        traversal.exit(key, version);
    }
}

#[derive(Clone, Copy)]
pub(in super::super) struct MutationInfo<'a> {
    pub(in super::super) metadata: mutation_metadata::MutationMetadata,
    pub(in super::super) assigned_member: Option<&'a str>,
    pub(in super::super) completed_at: u32,
    pub(in super::super) has_assignment: bool,
    pub(in super::super) proof: Option<&'a mutable_deferred::DeferredProof>,
}

impl ImportCollector {
    pub(in super::super) fn projected_receiver_is_terminal(
        &mut self,
        key: &(usize, String),
        has_root_read: bool,
    ) -> bool {
        let terminal = has_root_read
            && !self.prior_member_mutations.has_assigned_origins()
            && self.mutation_reference_is_terminal(key);
        #[cfg(feature = "test-instrumentation")]
        if terminal {
            self.value_flow.reference_terminal_paths += 1;
        }
        terminal
    }
}

#[cfg(feature = "test-instrumentation")]
impl ImportCollector {
    pub(in super::super) fn record_projection_counts<R: nested::RootCoordinate>(
        &mut self,
        traversal: receiver_traversal::ReceiverTraversal,
        binding_visits: usize,
        capacity: usize,
    ) {
        traversal.record_counts(&mut self.value_flow);
        self.value_flow.reference_binding_visits += binding_visits;
        self.value_flow.reference_pending_capacity += capacity;
        self.value_flow.reference_pending_bytes +=
            capacity * std::mem::size_of::<mutation_projection::PendingMutation<R>>();
    }
}
