use serde_yaml::Value;
use std::collections::BTreeMap;

pub(super) fn count_unexcluded(
    axes: &[(String, Vec<Value>)],
    exclusions: &[serde_yaml::Mapping],
    index: usize,
    values: &mut BTreeMap<String, Value>,
    limit: usize,
    states_remaining: &mut usize,
) -> Option<usize> {
    if index > super::MAX_STATIC_MATRIX_AXIS_DEPTH {
        return None;
    }
    if !consume_state(states_remaining) {
        return None;
    }
    if exclusions
        .iter()
        .any(|exclusion| exclusion_matches_assigned(exclusion, values))
    {
        return Some(0);
    }
    let Some((name, choices)) = axes.get(index) else {
        return Some(usize::from(!axes.is_empty()));
    };
    let mut count = 0_usize;
    for choice in choices {
        values.insert(name.clone(), choice.clone());
        count += count_unexcluded(
            axes,
            exclusions,
            index + 1,
            values,
            limit - count,
            states_remaining,
        )?;
        if count >= limit {
            break;
        }
    }
    values.remove(name);
    Some(count)
}

pub(super) fn exclusion_matches_assigned(
    exclusion: &serde_yaml::Mapping,
    values: &BTreeMap<String, Value>,
) -> bool {
    exclusion.iter().all(|(name, value)| {
        name.as_str()
            .and_then(|name| values.get(name))
            .is_some_and(|actual| actual == value)
    })
}

pub(super) fn has_applicable_combination(
    axes: &[(String, Vec<Value>)],
    exclusions: &[serde_yaml::Mapping],
    include: &serde_yaml::Mapping,
    index: usize,
    values: &mut BTreeMap<String, Value>,
    states_remaining: &mut usize,
) -> Option<bool> {
    if index > super::MAX_STATIC_MATRIX_AXIS_DEPTH {
        return None;
    }
    if !consume_state(states_remaining) {
        return None;
    }
    if exclusions
        .iter()
        .any(|exclusion| exclusion_matches_assigned(exclusion, values))
    {
        return Some(false);
    }
    let Some((name, choices)) = axes.get(index) else {
        return Some(!axes.is_empty());
    };
    let included_value = include.get(name);
    for choice in choices {
        if included_value.is_some_and(|included| included != choice) {
            continue;
        }
        values.insert(name.clone(), choice.clone());
        let applicable = has_applicable_combination(
            axes,
            exclusions,
            include,
            index + 1,
            values,
            states_remaining,
        )?;
        values.remove(name);
        if applicable {
            return Some(true);
        }
    }
    Some(false)
}

pub(super) fn consume_state(states_remaining: &mut usize) -> bool {
    let Some(next) = states_remaining.checked_sub(1) else {
        return false;
    };
    *states_remaining = next;
    true
}
