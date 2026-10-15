use super::*;

type Paths = FxHashMap<((usize, String), ValueMutationPath), Vec<nested::ReceiverRead>>;
#[derive(Default)]
struct TraversalState {
    active: FxHashSet<(usize, String)>,
    visited: Paths,
}
#[derive(Default)]
pub(super) struct ReceiverTraversal {
    ordinary: TraversalState,
    mutable: Option<Box<FxHashMap<mutable_origins::RootVersion, TraversalState>>>,
}
impl ReceiverTraversal {
    pub(super) fn enter(
        &mut self,
        key: &(usize, String),
        path: &[Option<String>],
        reads: Vec<nested::ReceiverRead>,
        version: Option<mutable_origins::RootVersion>,
    ) -> Option<Vec<nested::ReceiverRead>> {
        let state = match version {
            Some(version) => self
                .mutable
                .get_or_insert_with(Default::default)
                .entry(version)
                .or_default(),
            None => &mut self.ordinary,
        };
        if state.active.contains(key) {
            return None;
        }
        let reads = merge_reads(&mut state.visited, key, path, reads)?;
        state.active.insert(key.clone());
        Some(reads)
    }
    pub(super) fn exit(
        &mut self,
        key: &(usize, String),
        version: Option<mutable_origins::RootVersion>,
    ) {
        let state = match version {
            Some(version) => self
                .mutable
                .as_mut()
                .and_then(|states| states.get_mut(&version)),
            None => Some(&mut self.ordinary),
        };
        if let Some(state) = state {
            state.active.remove(key);
        }
    }
}

/// One monotone cutoff summary per receiver/path, rather than one state per
/// acquisition route. An ambiguous context can never prove replacement.
fn merge_reads(
    visited: &mut Paths,
    key: &(usize, String),
    path: &[Option<String>],
    reads: Vec<nested::ReceiverRead>,
) -> Option<Vec<nested::ReceiverRead>> {
    let state = (key.clone(), path.to_vec());
    let Some(previous) = visited.get_mut(&state) else {
        visited.insert(state, reads.clone());
        return Some(reads);
    };
    let mut changed = false;
    for (previous, incoming) in previous.iter_mut().zip(&reads) {
        let merged = match (*previous, *incoming) {
            (Some((left, context)), Some((right, other))) if context == other => {
                Some((left.min(right), context))
            }
            _ => None,
        };
        changed |= *previous != merged;
        *previous = merged;
    }
    changed.then(|| previous.clone())
}
