use super::Cols;
use std::collections::BTreeSet;

pub(super) fn union(mut left: Cols, right: Cols) -> Cols {
    left.proven.extend(right.proven);
    left.pending.extend(right.pending);
    left.pending.retain(|name| !left.proven.contains(name));
    left
}

pub(super) fn intersect(left: Cols, right: Cols) -> Cols {
    let mut proven = BTreeSet::new();
    let mut pending = BTreeSet::new();
    for name in left.proven.union(&left.pending) {
        let on_right = right.proven.contains(name) || right.pending.contains(name);
        if !on_right {
            continue;
        }
        if left.proven.contains(name) && right.proven.contains(name) {
            proven.insert(name.clone());
        } else {
            pending.insert(name.clone());
        }
    }
    Cols { proven, pending }
}
