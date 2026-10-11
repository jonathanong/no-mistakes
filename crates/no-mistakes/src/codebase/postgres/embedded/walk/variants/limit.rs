use super::Recovered;
use crate::codebase::postgres::embedded::MAX_EMBEDDED_SQL_VARIANTS;

pub(super) fn bounded(values: Vec<Recovered>) -> Option<Vec<Recovered>> {
    let mut unique = Vec::new();
    for mut value in values {
        normalize(&mut value);
        // Path extension and composition reject incompatible choices before
        // values arrive here; this boundary only normalizes and caps them.
        let index = unique
            .iter()
            .position(|other: &Recovered| other.same_value(&value))
            .unwrap_or_else(|| {
                unique.push(value.clone());
                unique.len() - 1
            });
        unique[index].merge_append_sites(&value);
        if unique.len() > MAX_EMBEDDED_SQL_VARIANTS {
            return None;
        }
    }
    Some(unique)
}

/// A binary decision that produces the same physical value on both arms adds
/// no constraint. Fold it where both arms are known to be exhaustive: switch
/// choices can have more than two arms and must not use this simplification.
pub(super) fn binary_alternatives(
    left: Vec<Recovered>,
    mut right: Vec<Recovered>,
    id: u64,
) -> Option<Vec<Recovered>> {
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
            value.merge_append_sites(&other);
        }
        values.push(value);
    }
    values.extend(right);
    bounded(values)
}

fn normalize(value: &mut Recovered) {
    value.choices.sort_unstable();
    value.choices.dedup();
}

fn same_without_choice(left: &Recovered, right: &Recovered, id: u64) -> bool {
    let mut left = left.clone();
    let mut right = right.clone();
    left.choices.retain(|(other, _)| *other != id);
    right.choices.retain(|(other, _)| *other != id);
    // Whether an alternative was previously enumerated is bookkeeping, not
    // part of its physical SQL identity. Preserve it when the pair is folded.
    left.enumerated = false;
    right.enumerated = false;
    normalize(&mut right);
    left.same_value(&right)
}
