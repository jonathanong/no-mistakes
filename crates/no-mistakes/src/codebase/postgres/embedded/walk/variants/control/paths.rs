use super::Snapshot;

/// Keep prior alternatives only on branch paths that can actually reach them.
/// Unchanged values gain no new choices, so unrelated branches do not consume
/// the version budget of a later SQL call.
pub(super) fn restrict(snapshot: &Snapshot, paths: &[Vec<(u64, u32)>]) -> Snapshot {
    // Empty execution paths already disable recovery in the arm. Keep its
    // entry bindings so merely visiting an unreachable arm does not look
    // like a mutation of every binding; actual writes still change their own.
    if paths.is_empty() {
        return snapshot.clone();
    }
    snapshot
        .iter()
        .map(|scope| {
            scope
                .iter()
                .map(|(name, values)| {
                    let values = values.as_ref().map(|values| {
                        values
                            .iter()
                            .filter(|value| {
                                paths.iter().any(|path| {
                                    !value.choices.iter().any(|(id, arm)| {
                                        path.iter()
                                            .any(|(other, choice)| id == other && arm != choice)
                                    })
                                })
                            })
                            .cloned()
                            .collect()
                    });
                    (name.clone(), values)
                })
                .collect()
        })
        .collect()
}

pub(super) fn taken(left: &[Vec<(u64, u32)>], right: &[Vec<(u64, u32)>]) -> Option<bool> {
    match (left.is_empty(), right.is_empty()) {
        (false, true) => Some(true),
        (true, false) => Some(false),
        _ => None,
    }
}

pub(in crate::codebase::postgres::embedded::walk) fn branch_paths(
    paths: &[Vec<(u64, u32)>],
    id: u64,
    arm: u32,
) -> Vec<Vec<(u64, u32)>> {
    paths
        .iter()
        .filter(|path| {
            !path
                .iter()
                .any(|(other, choice)| *other == id && *choice != arm)
        })
        .map(|path| {
            let mut path = path.clone();
            if !path.contains(&(id, arm)) {
                path.push((id, arm));
            }
            path
        })
        .collect()
}
