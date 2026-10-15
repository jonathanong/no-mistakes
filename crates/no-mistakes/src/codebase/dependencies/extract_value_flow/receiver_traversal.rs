use super::*;

#[path = "assigned_receiver_traversal.rs"]
mod assigned;
#[path = "receiver_traversal_state.rs"]
mod state;
use state::TraversalState;

#[path = "receiver_traversal_owned_entry.rs"]
mod owned_entry;

struct MutableTraversal {
    first: (mutable_origins::RootVersion, TraversalState),
    other: FxHashMap<mutable_origins::RootVersion, TraversalState>,
}
impl MutableTraversal {
    fn state(&mut self, version: mutable_origins::RootVersion) -> &mut TraversalState {
        if self.first.0 == version {
            &mut self.first.1
        } else {
            self.other.entry(version).or_default()
        }
    }
    fn existing(&mut self, version: mutable_origins::RootVersion) -> Option<&mut TraversalState> {
        if self.first.0 == version {
            Some(&mut self.first.1)
        } else {
            self.other.get_mut(&version)
        }
    }
}
#[derive(Default)]
pub(super) struct ReceiverTraversal {
    ordinary: TraversalState,
    assigned: Option<Box<assigned::AssignedTraversal>>,
    #[cfg(feature = "test-instrumentation")]
    pub(super) version_dispatches: usize,
    mutable: Option<MutableTraversal>,
}
impl ReceiverTraversal {
    pub(super) fn enter_assigned(
        &mut self,
        key: &(usize, String),
        path: &[Option<String>],
        reads: Vec<nested::ReceiverRead>,
        version: Option<mutable_origins::RootVersion>,
        receipts: assigned_receiver_origins::Receipts,
    ) -> Option<Vec<nested::ReceiverRead>> {
        self.assigned
            .get_or_insert_with(Default::default)
            .enter(key, path, reads, version, receipts)
    }
    pub(super) fn exit_assigned(&mut self, key: &(usize, String)) {
        self.assigned
            .as_mut()
            .expect("active RHS traversal")
            .exit(key);
    }

    pub(super) fn exit_ordinary(&mut self, key: &(usize, String)) {
        self.ordinary.remove_active(key);
    }

    /// An exit belongs to its sole active version; ambiguous states still
    /// resolve the original source coordinate before removing their key.
    pub(super) fn exit_single_version(&mut self, key: &(usize, String)) -> bool {
        let Some(states) = self.mutable.as_mut() else {
            return false;
        };
        states.other.is_empty()
            && !self.ordinary.contains_active(key)
            && states.first.1.remove_active(key)
    }

    #[cfg(feature = "test-instrumentation")]
    pub(super) fn record_counts(&self, facts: &mut ValueFlowFacts) {
        facts.reference_version_dispatches += self.version_dispatches;
        facts.reference_version_table_capacity += self.version_table_capacity();
        facts.reference_state_heap_buffers += self.ordinary.heap_buffers();
        if let Some(states) = &self.mutable {
            facts.reference_state_heap_buffers += states.first.1.heap_buffers()
                + states
                    .other
                    .values()
                    .map(TraversalState::heap_buffers)
                    .sum::<usize>();
        }
    }

    #[cfg(feature = "test-instrumentation")]
    pub(super) fn version_table_capacity(&self) -> usize {
        self.mutable
            .as_ref()
            .map_or(0, |state| state.other.capacity())
    }

    pub(super) fn exit(
        &mut self,
        key: &(usize, String),
        version: Option<mutable_origins::RootVersion>,
    ) {
        #[cfg(feature = "test-instrumentation")]
        {
            self.version_dispatches += 1;
        }
        let state = match version {
            Some(version) => self
                .mutable
                .as_mut()
                .and_then(|states| states.existing(version)),
            None => Some(&mut self.ordinary),
        };
        if let Some(state) = state {
            state.remove_active(key);
        }
    }
}

#[cfg(test)]
#[path = "receiver_traversal_tests.rs"]
mod tests;

include!("receiver_successor_classification.rs");

#[cfg(test)]
#[path = "receiver_traversal_inline_tests.rs"]
mod inline_tests;

#[cfg(test)]
#[path = "receiver_traversal_differential_tests.rs"]
mod differential_tests;

#[cfg(test)]
#[path = "receiver_traversal_oracle_tests.rs"]
mod oracle;
