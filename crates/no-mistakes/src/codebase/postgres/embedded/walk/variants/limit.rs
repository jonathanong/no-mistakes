use super::Recovered;
use crate::codebase::postgres::embedded::MAX_EMBEDDED_SQL_VARIANTS;

pub(super) fn bounded(values: Vec<Recovered>) -> Option<Vec<Recovered>> {
    // Saved fixtures and observer counters check the exploration budget.
    // This boundary executes the same algorithm in ordinary and test builds.
    let mut unique: Vec<Recovered> = Vec::new();
    for mut value in values {
        normalize(&mut value);
        // Path extension and composition reject incompatible choices before
        // values arrive here; this boundary only normalizes and caps them.
        if let Some(other) = unique.iter_mut().find(|other| other.same_value(&value)) {
            other.merge_alternative(&value);
            normalize(other);
        } else {
            unique.push(value);
        }
    }
    if unique.len() > 1 && unique.iter().any(|value| !value.append_sites.is_empty()) {
        // Prune each join. Conditional cardinality is independent of how many
        // append calls one branch executes or how many unconditional calls ran.
        unique.sort_by(|left, right| {
            left.append_decisions
                .len()
                .cmp(&right.append_decisions.len())
                .then_with(|| left.append_sites.cmp(&right.append_sites))
                .then_with(|| left.append_decisions.cmp(&right.append_decisions))
        });
        if unique.len() > MAX_EMBEDDED_SQL_VARIANTS {
            let decisions: crate::fx::FxHashSet<_> = unique
                .iter()
                .flat_map(|value| {
                    value
                        .choices
                        .iter()
                        .map(|(id, _)| *id)
                        .chain(value.append_decisions.iter().copied())
                })
                .collect();
            let cap = MAX_EMBEDDED_SQL_VARIANTS.max(decisions.len() + 1);
            if unique.len() > cap {
                unique = select_witnesses(unique, cap);
                for value in &mut unique {
                    value.variants_truncated = true;
                }
            }
        }
    } else if unique.len() > MAX_EMBEDDED_SQL_VARIANTS {
        return None;
    }
    Some(unique)
}

fn select_witnesses(values: Vec<Recovered>, cap: usize) -> Vec<Recovered> {
    let mut retained = vec![false; values.len()];
    let mut covered = crate::fx::FxHashSet::default();
    let mut count = 0;
    // Sorted input gives every decision its smallest feasible witness. A nested
    // decision needs its parent guards; that witness outranks unrelated pairs.
    for (index, value) in values.iter().enumerate() {
        if index == 0
            || value
                .append_decisions
                .iter()
                .any(|id| !covered.contains(id))
        {
            retained[index] = true;
            count += 1;
            covered.extend(value.append_decisions.iter().copied());
        }
    }
    debug_assert!(count <= cap);
    let mut primary: Vec<usize> = Vec::new();
    let mut duplicate_paths = Vec::new();
    for (index, value) in values.iter().enumerate() {
        // Preserve every constraint as its own path, but prefer a distinct SQL
        // outcome over a second all-off path through nested no-op branches.
        if primary.iter().map(|index| &values[*index]).any(|other| {
            other.append_decisions == value.append_decisions && other.same_physical_value(value)
        }) {
            duplicate_paths.push(index);
        } else {
            primary.push(index);
        }
    }
    primary.extend(duplicate_paths);
    for index in primary {
        if count == cap {
            break;
        }
        if !retained[index] {
            retained[index] = true;
            count += 1;
        }
    }
    values
        .into_iter()
        .enumerate()
        .filter_map(|(index, value)| retained[index].then_some(value))
        .collect()
}

/// A binary decision that produces the same physical value on both arms adds
/// no constraint. Fold it where both arms are known to be exhaustive: switch
/// choices can have more than two arms and must not use this simplification.
pub(super) fn binary_alternatives(
    left: Vec<Recovered>,
    mut right: Vec<Recovered>,
    id: u64,
) -> Option<Vec<Recovered>> {
    right.iter_mut().for_each(sort_choices);
    let mut values = Vec::new();
    for mut value in left {
        normalize(&mut value);
        // Conditional recovery attaches opposite arms of this decision to
        // left and right. All remaining choices must still agree exactly.
        let matched = right
            .iter()
            .position(|other| same_without_choice(&value, other, id));
        if let Some(index) = matched {
            let other = right.remove(index);
            value.choices.retain(|(other, _)| *other != id);
            value.enumerated |= other.enumerated;
            value.merge_alternative(&other);
            // Removing the last choice must not collapse a recovered append
            // back to legacy text and discard its physical contributing sites.
            value.enumerated |= !value.append_sites.is_empty();
            value.append_decisions.retain(|other| *other != id);
        }
        values.push(value);
    }
    values.extend(right);
    bounded(values)
}

fn sort_choices(value: &mut Recovered) {
    value.choices.sort_unstable();
    value.choices.dedup();
}

fn normalize(value: &mut Recovered) {
    sort_choices(value);
    value.decision_offs.sort_unstable();
    value.decision_offs.dedup_by_key(|(id, _, _)| *id);
    value.append_decisions = value
        .decision_offs
        .iter()
        .filter_map(|(id, _, off)| {
            value
                .choices
                .iter()
                .any(|(other, arm)| id == other && arm != off)
                .then_some(*id)
        })
        .collect();
}

fn same_without_choice(left: &Recovered, right: &Recovered, id: u64) -> bool {
    left.same_physical_value(right)
        && left
            .choices
            .iter()
            .filter(|(choice, _)| *choice != id)
            .eq(right.choices.iter().filter(|(choice, _)| *choice != id))
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "limit/tests.rs"]
mod tests;
