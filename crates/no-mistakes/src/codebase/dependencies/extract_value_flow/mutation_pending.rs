use super::*;

pub(super) fn append<R: nested::RootCoordinate>(
    pending: &mut Vec<PendingMutation<R>>,
    exit_position: usize,
    key: (usize, String),
    root_read: R,
    assigned_exit: bool,
    references: reference_output::ReferenceOutput<nested::ReceiverWrite<R>>,
) {
    let references = references
        .into_iter()
        .map(|reference| (reference, false, true, true));
    // The old exit occupied this slot before successors were appended. Retain
    // its capacity headroom as well as its LIFO position, including wide fans.
    pending.reserve(references.size_hint().0.saturating_add(1));
    pending.extend(references);
    pending.insert(
        exit_position,
        (
            ((key, Vec::new()), Vec::new(), root_read),
            true,
            false,
            assigned_exit,
        ),
    );
}
