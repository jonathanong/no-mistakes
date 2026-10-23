use super::super::{ArgumentObjects, Scope, Value};
use crate::fx::{FxHashMap, FxHashSet};
use std::collections::BTreeMap;

pub(super) fn changed_argument_ids(
    before_objects: &ArgumentObjects,
    after_objects: &ArgumentObjects,
    before_extras: &FxHashMap<u64, BTreeMap<usize, Value>>,
    after_extras: &FxHashMap<u64, BTreeMap<usize, Value>>,
) -> FxHashSet<u64> {
    let mut changed = FxHashSet::default();
    for (id, slots) in before_objects {
        if after_objects.get(id).is_none_or(|after| after != slots) {
            changed.insert(*id);
        }
    }
    for id in after_objects.keys() {
        if !before_objects.contains_key(id) {
            changed.insert(*id);
        }
    }
    for (id, slots) in before_extras {
        if after_extras.get(id) != Some(slots) {
            changed.insert(*id);
        }
    }
    for id in after_extras.keys() {
        if !before_extras.contains_key(id) {
            changed.insert(*id);
        }
    }
    // A slot that still points at B does not change A's identity, but B's
    // arena is visible through that slot. Parents of a changed id stay live.
    if !changed.is_empty() {
        close_argument_parents(after_objects, after_extras, &mut changed);
    }
    changed
}

fn close_argument_parents(
    objects: &ArgumentObjects,
    extras: &FxHashMap<u64, BTreeMap<usize, Value>>,
    changed: &mut FxHashSet<u64>,
) {
    let mut parents: FxHashMap<u64, Vec<u64>> = FxHashMap::default();
    let mut link = |owner: u64, arguments: &FxHashSet<u64>| {
        for child in arguments {
            if *child != owner {
                parents.entry(*child).or_default().push(owner);
            }
        }
    };
    for (id, slots) in objects {
        link(*id, &slots.references().arguments);
    }
    for (id, slots) in extras {
        link(
            *id,
            &super::super::references::Edges::collect(slots.values()).arguments,
        );
    }
    let mut queue: Vec<u64> = changed.iter().copied().collect();
    while let Some(id) = queue.pop() {
        let Some(owners) = parents.get(&id) else {
            continue;
        };
        for owner in owners.clone() {
            if changed.insert(owner) {
                queue.push(owner);
            }
        }
    }
}

/// An untouched scope Arc still observes arena writes through argument ids.
pub(super) fn unchanged_shared_scope(
    before: &Scope,
    after: &Scope,
    changed_arguments: &FxHashSet<u64>,
) -> bool {
    if !before.same_bindings(after) {
        return false;
    }
    changed_arguments.is_empty() || before.references().arguments.is_disjoint(changed_arguments)
}
