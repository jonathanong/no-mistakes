use super::traversal::{consume_state, exclusion_matches_assigned};
use super::{
    static_mappings, static_matrix_axes, StaticMappings, StaticMatrixAxes,
    STATIC_MATRIX_ENUMERATION_LIMIT,
};
use serde_yaml::{Mapping, Value};
use std::collections::BTreeMap;

pub(super) fn values(job: &Value) -> BTreeMap<String, Value> {
    let Some(matrix) = job
        .get("strategy")
        .and_then(|strategy| strategy.get("matrix"))
        .and_then(Value::as_mapping)
    else {
        return BTreeMap::new();
    };
    let StaticMatrixAxes::Static(axes) = static_matrix_axes(matrix) else {
        return BTreeMap::new();
    };
    let StaticMappings::Static(exclusions) = static_mappings(matrix.get("exclude")) else {
        return BTreeMap::new();
    };
    let StaticMappings::Static(includes) = static_mappings(matrix.get("include")) else {
        return BTreeMap::new();
    };
    let mut originals = Vec::new();
    let mut assigned = BTreeMap::new();
    let mut states_remaining = STATIC_MATRIX_ENUMERATION_LIMIT;
    if !collect_combinations(
        &axes,
        &exclusions,
        0,
        &mut assigned,
        &mut originals,
        &mut states_remaining,
    ) {
        return BTreeMap::new();
    }
    let mut combinations = originals.clone();
    for include in includes {
        let mut applied = false;
        for (index, original) in originals.iter().enumerate() {
            if include_compatible(include, original, &axes) {
                for (name, value) in include {
                    let Some(name) = name.as_str() else {
                        return BTreeMap::new();
                    };
                    combinations[index].insert(name.to_string(), value.clone());
                }
                applied = true;
            }
        }
        if !applied {
            let Some(included) = include
                .iter()
                .map(|(name, value)| Some((name.as_str()?.to_string(), value.clone())))
                .collect::<Option<BTreeMap<_, _>>>()
            else {
                return BTreeMap::new();
            };
            combinations.push(included);
        }
    }
    uniform_values(&combinations)
}

fn collect_combinations(
    axes: &[(String, Vec<Value>)],
    exclusions: &[&Mapping],
    index: usize,
    assigned: &mut BTreeMap<String, Value>,
    combinations: &mut Vec<BTreeMap<String, Value>>,
    states_remaining: &mut usize,
) -> bool {
    if !consume_state(states_remaining) {
        return false;
    }
    if exclusions
        .iter()
        .any(|exclusion| exclusion_matches_assigned(exclusion, assigned))
    {
        return true;
    }
    let Some((name, choices)) = axes.get(index) else {
        if !axes.is_empty() {
            combinations.push(assigned.clone());
        }
        return true;
    };
    for choice in choices {
        assigned.insert(name.clone(), choice.clone());
        if !collect_combinations(
            axes,
            exclusions,
            index + 1,
            assigned,
            combinations,
            states_remaining,
        ) {
            return false;
        }
    }
    assigned.remove(name);
    true
}

fn include_compatible(
    include: &Mapping,
    original: &BTreeMap<String, Value>,
    axes: &[(String, Vec<Value>)],
) -> bool {
    axes.iter().all(|(name, _)| {
        include
            .get(name)
            .is_none_or(|value| original.get(name) == Some(value))
    })
}

fn uniform_values(combinations: &[BTreeMap<String, Value>]) -> BTreeMap<String, Value> {
    let Some(first) = combinations.first() else {
        return BTreeMap::new();
    };
    first
        .iter()
        .filter(|(name, value)| {
            static_scalar(value)
                && combinations
                    .iter()
                    .all(|combination| combination.get(*name) == Some(*value))
        })
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect()
}

fn static_scalar(value: &Value) -> bool {
    matches!(value, Value::Bool(_) | Value::Number(_) | Value::Null)
        || value
            .as_str()
            .is_some_and(|value| !value.trim().starts_with("${{") && !value.trim().ends_with("}}"))
}
