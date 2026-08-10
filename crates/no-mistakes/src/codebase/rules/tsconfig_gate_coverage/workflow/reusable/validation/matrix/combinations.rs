use super::{
    static_mappings, static_matrix_axes, StaticMappings, StaticMatrixAxes,
    STATIC_MATRIX_ENUMERATION_LIMIT,
};
use serde_yaml::Value;
use std::collections::BTreeMap;

pub(in super::super::super) fn static_matrix_combinations(
    job: &Value,
) -> Option<Vec<BTreeMap<String, Value>>> {
    let Some(matrix) = job
        .get("strategy")
        .and_then(|strategy| strategy.get("matrix"))
    else {
        return Some(vec![BTreeMap::new()]);
    };
    let Some(matrix) = matrix.as_mapping() else {
        return super::super::super::super::complete_expression(matrix.as_str()?)
            .then(|| vec![BTreeMap::new()]);
    };
    let axes = match static_matrix_axes(matrix) {
        StaticMatrixAxes::Static(axes) => axes,
        StaticMatrixAxes::Dynamic => return Some(vec![BTreeMap::new()]),
        StaticMatrixAxes::Invalid => return None,
    };
    let exclusions = match static_mappings(matrix.get("exclude")) {
        StaticMappings::Static(exclusions) => exclusions,
        StaticMappings::Dynamic => return Some(vec![BTreeMap::new()]),
        StaticMappings::Invalid => return None,
    };
    let includes = match static_mappings(matrix.get("include")) {
        StaticMappings::Static(includes) => includes,
        StaticMappings::Dynamic => return Some(vec![BTreeMap::new()]),
        StaticMappings::Invalid => return None,
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
        return None;
    }
    let mut combinations = originals.clone();
    for include in includes {
        let mut applied = false;
        for (index, original) in originals.iter().enumerate() {
            if include_compatible(include, original, &axes) {
                for (name, value) in include {
                    combinations[index].insert(name.as_str()?.to_string(), value.clone());
                }
                applied = true;
            }
        }
        if !applied {
            let included = include
                .iter()
                .map(|(name, value)| Some((name.as_str()?.to_string(), value.clone())))
                .collect::<Option<BTreeMap<_, _>>>()?;
            combinations.push(included);
        }
    }
    Some(combinations)
}

fn collect_combinations(
    axes: &[(String, Vec<Value>)],
    exclusions: &[&serde_yaml::Mapping],
    index: usize,
    assigned: &mut BTreeMap<String, Value>,
    combinations: &mut Vec<BTreeMap<String, Value>>,
    states_remaining: &mut usize,
) -> bool {
    if !super::traversal::consume_state(states_remaining) {
        return false;
    }
    if exclusions
        .iter()
        .any(|exclusion| super::traversal::exclusion_matches_assigned(exclusion, assigned))
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
    include: &serde_yaml::Mapping,
    original: &BTreeMap<String, Value>,
    axes: &[(String, Vec<Value>)],
) -> bool {
    axes.iter().all(|(name, _)| {
        include
            .get(name)
            .is_none_or(|value| original.get(name) == Some(value))
    })
}
