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
    changed
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
