use super::*;

#[path = "receiver_traversal_state.rs"]
mod state;
use state::TraversalState;

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
    #[cfg(feature = "test-instrumentation")]
    pub(super) version_dispatches: usize,
    mutable: Option<MutableTraversal>,
}
impl ReceiverTraversal {
    pub(super) fn enter_ordinary(
        &mut self,
        key: &(usize, String),
        path: &[Option<String>],
        reads: Vec<nested::ReceiverRead>,
    ) -> Option<Vec<nested::ReceiverRead>> {
        self.ordinary.enter(key, path, reads)
    }
    pub(super) fn exit_ordinary(&mut self, key: &(usize, String)) {
        self.ordinary.active.remove(key);
    }

    /// An exit belongs to its sole active version; ambiguous states still
    /// resolve the original source coordinate before removing their key.
    pub(super) fn exit_single_version(&mut self, key: &(usize, String)) -> bool {
        let Some(states) = self.mutable.as_mut() else {
            return false;
        };
        states.other.is_empty()
            && !self.ordinary.active.contains(key)
            && states.first.1.active.remove(key)
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

    pub(super) fn enter(
        &mut self,
        key: &(usize, String),
        path: &[Option<String>],
        reads: Vec<nested::ReceiverRead>,
        version: Option<mutable_origins::RootVersion>,
    ) -> Option<Vec<nested::ReceiverRead>> {
        #[cfg(feature = "test-instrumentation")]
        {
            self.version_dispatches += 1;
        }
        let state = match version {
            Some(version) => self
                .mutable
                .get_or_insert_with(|| MutableTraversal {
                    first: (version, TraversalState::default()),
                    other: fx_map(),
                })
                .state(version),
            None => &mut self.ordinary,
        };
        state.enter(key, path, reads)
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
            state.active.remove(key);
        }
    }
}

#[cfg(test)]
#[path = "receiver_traversal_tests.rs"]
mod tests;

impl ImportCollector {
    pub(super) fn mutation_reference_is_terminal(&self, key: &(usize, String)) -> bool {
        self.value_binding_indexes
            .get(&key.0)
            .and_then(|names| names.get(key.1.as_str()))
            .is_none_or(Vec::is_empty)
            && self
                .value_reference_links
                .as_ref()
                .and_then(|links| links.get(&key.0))
                .and_then(|names| names.get(key.1.as_str()))
                .is_none_or(|links| links.is_empty())
            && !self.has_mutable_receiver_binding(key.0, &key.1)
    }
}
