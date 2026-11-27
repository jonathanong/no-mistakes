use super::*;

type Version = (
    Option<mutable_origins::RootVersion>,
    assigned_receiver_origins::Receipts,
);
/// RHS receipts qualify the same canonical traversal; ordinary writes never
/// allocate this state. The stack follows its existing depth-first exit tasks.
#[derive(Default)]
pub(super) struct AssignedTraversal {
    versions: FxHashMap<Version, usize>,
    states: Vec<AssignedState>,
    active: Vec<(usize, Option<usize>)>,
}
#[derive(Default)]
struct AssignedState {
    active: FxHashMap<(usize, String), usize>,
    visited: state::TraversalState,
}
impl AssignedTraversal {
    pub(super) fn enter(
        &mut self,
        key: &(usize, String),
        path: &[Option<String>],
        reads: Vec<nested::ReceiverRead>,
        version: Option<mutable_origins::RootVersion>,
        receipts: assigned_receiver_origins::Receipts,
    ) -> Option<Vec<nested::ReceiverRead>> {
        let index = *self.versions.entry((version, receipts)).or_insert_with(|| {
            self.states.push(AssignedState::default());
            self.states.len() - 1
        });
        let state = &mut self.states[index];
        let previous = state.active.get(key).copied();
        // A finite path may traverse a self-reference while consuming members.
        // Equal or growing suffixes still stop a non-progressing alias cycle.
        if previous.is_some_and(|depth| path.len() >= depth) {
            return None;
        }
        let reads = state.visited.enter(key, path, reads)?;
        // RHS cycles use receipt-qualified path depth above. Release the
        // canonical binding-only activity while retaining its cutoff selection.
        state.visited.remove_active(key);
        state.active.insert(key.clone(), path.len());
        self.active.push((index, previous));
        Some(reads)
    }
    pub(super) fn exit(&mut self, key: &(usize, String)) {
        let (index, previous) = self.active.pop().expect("matching RHS traversal entry");
        if let Some(previous) = previous {
            self.states[index].active.insert(key.clone(), previous);
        } else {
            self.states[index].active.remove(key);
        }
    }
}
