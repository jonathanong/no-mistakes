//! Recheck only items whose pin dependencies acquired a bounded proof.
use super::keys::Keys;
use std::collections::VecDeque;

pub(super) fn bound(keys: &[Keys], bounded: &mut [bool]) {
    let mut dependents = vec![Vec::new(); keys.len()];
    for (index, keys) in keys.iter().enumerate() {
        for dependency in keys.iter().flatten().flatten().flatten() {
            dependents[*dependency].push(index);
        }
    }
    for dependents in &mut dependents {
        dependents.sort_unstable();
        dependents.dedup();
    }
    let mut queued = vec![true; keys.len()];
    let mut queue: VecDeque<_> = (0..keys.len()).collect();
    while let Some(index) = queue.pop_front() {
        queued[index] = false;
        if bounded[index]
            || !keys[index].iter().any(|key| {
                key.iter().all(|column| {
                    column
                        .iter()
                        .any(|source| source.iter().all(|other| bounded[*other]))
                })
            })
        {
            continue;
        }
        bounded[index] = true;
        for &dependent in &dependents[index] {
            if !bounded[dependent] && !queued[dependent] {
                queue.push_back(dependent);
                queued[dependent] = true;
            }
        }
    }
}
