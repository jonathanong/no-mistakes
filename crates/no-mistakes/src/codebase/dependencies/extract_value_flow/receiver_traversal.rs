use super::*;

type Paths = FxHashMap<((usize, String), ValueMutationPath), Vec<nested::ReceiverRead>>;

/// One monotone cutoff summary per receiver/path, rather than one state per
/// acquisition route. An ambiguous context can never prove replacement.
pub(super) fn merge_reads(
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
