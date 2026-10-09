use super::{super::Value, merge};
use crate::fx::FxHashMap;
use std::collections::BTreeMap;

pub(super) fn join(
    joined: &mut Option<FxHashMap<u64, BTreeMap<usize, Value>>>,
    current: &FxHashMap<u64, BTreeMap<usize, Value>>,
    original_ids: &FxHashMap<u64, Vec<Value>>,
) {
    let Some(joined) = joined else {
        *joined = Some(current.clone());
        return;
    };
    for (id, slots) in current {
        // Arm-created module containers can persist into subsequent arms.
        // Only a genuinely new identity starts with its current slot state.
        if !joined.contains_key(id) {
            joined.insert(*id, slots.clone());
            continue;
        }
        let previous = joined.entry(*id).or_default();
        let indices = previous
            .keys()
            .chain(slots.keys())
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        for index in indices {
            let before = previous.get(&index).cloned().unwrap_or(Value::Unknown);
            let after = slots.get(&index).cloned().unwrap_or(Value::Unknown);
            previous.insert(index, merge::value(before, after));
        }
    }
    for (id, slots) in joined.iter_mut() {
        if original_ids.contains_key(id) && !current.contains_key(id) {
            for value in slots.values_mut() {
                *value = merge::value(value.clone(), Value::Unknown);
            }
        }
    }
}
