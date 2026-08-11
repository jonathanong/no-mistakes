use super::{
    static_mappings, static_matrix_axes, static_matrix_job_count, StaticMappings, StaticMatrixAxes,
    StaticMatrixJobCount, MATRIX_JOB_LIMIT, STATIC_MATRIX_ENUMERATION_LIMIT,
};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::InputState;
use serde_yaml::Value;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
pub(in super::super::super) enum MatrixCombinations {
    Static(Vec<BTreeMap<String, Value>>),
    Dynamic(Vec<BTreeMap<String, Value>>),
}

pub(in super::super::super) fn static_matrix_combinations_for_inputs(
    job: &Value,
    inputs: &InputState,
) -> Option<MatrixCombinations> {
    let Some(matrix) = job
        .get("strategy")
        .and_then(|strategy| strategy.get("matrix"))
    else {
        return Some(MatrixCombinations::Static(vec![BTreeMap::new()]));
    };
    let Some(expression) = matrix.as_str() else {
        return static_matrix_combinations_for_value(matrix);
    };
    match super::root_expression::resolve(expression, inputs) {
        super::root_expression::ResolvedRootMatrix::Mapping(matrix) => {
            static_matrix_combinations_for_mapping(&matrix)
        }
        super::root_expression::ResolvedRootMatrix::NonMapping => None,
        super::root_expression::ResolvedRootMatrix::Dynamic => {
            static_matrix_combinations_for_value(matrix)
        }
    }
}

fn static_matrix_combinations_for_value(matrix: &Value) -> Option<MatrixCombinations> {
    let Some(matrix) = matrix.as_mapping() else {
        return super::matrix_expression_may_be_mapping(matrix.as_str()?)
            .then_some(MatrixCombinations::Dynamic(vec![BTreeMap::new()]));
    };
    static_matrix_combinations_for_mapping(matrix)
}

fn static_matrix_combinations_for_mapping(
    matrix: &serde_yaml::Mapping,
) -> Option<MatrixCombinations> {
    match static_matrix_job_count(matrix) {
        StaticMatrixJobCount::Known(count) if count <= MATRIX_JOB_LIMIT => {}
        StaticMatrixJobCount::Dynamic => {}
        StaticMatrixJobCount::Known(_) | StaticMatrixJobCount::Invalid => return None,
    }
    let axes = match static_matrix_axes(matrix) {
        axes @ (StaticMatrixAxes::Static(_) | StaticMatrixAxes::Dynamic) => axes,
        StaticMatrixAxes::Invalid => return None,
    };
    let exclusions = match static_mappings(matrix.get("exclude")) {
        mappings @ (StaticMappings::Static(_) | StaticMappings::Dynamic) => mappings,
        StaticMappings::Invalid => return None,
    };
    let includes = match static_mappings(matrix.get("include")) {
        mappings @ (StaticMappings::Static(_) | StaticMappings::Dynamic) => mappings,
        StaticMappings::Invalid => return None,
    };
    let (axes, exclusions, includes) = match (axes, exclusions, includes) {
        (
            StaticMatrixAxes::Static(axes),
            StaticMappings::Static(exclusions),
            StaticMappings::Static(includes),
        ) => (axes, exclusions, includes),
        (StaticMatrixAxes::Dynamic, _, _)
        | (_, StaticMappings::Dynamic, _)
        | (_, _, StaticMappings::Dynamic) => {
            return Some(MatrixCombinations::Dynamic(vec![BTreeMap::new()]));
        }
        (StaticMatrixAxes::Invalid, _, _)
        | (_, StaticMappings::Invalid, _)
        | (_, _, StaticMappings::Invalid) => return None,
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
            if include_compatible(&include, original, &axes) {
                for (name, value) in &include {
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
    Some(MatrixCombinations::Static(combinations))
}

impl std::ops::Deref for MatrixCombinations {
    type Target = Vec<BTreeMap<String, Value>>;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Static(values) | Self::Dynamic(values) => values,
        }
    }
}

fn collect_combinations(
    axes: &[(String, Vec<Value>)],
    exclusions: &[serde_yaml::Mapping],
    index: usize,
    assigned: &mut BTreeMap<String, Value>,
    combinations: &mut Vec<BTreeMap<String, Value>>,
    states_remaining: &mut usize,
) -> bool {
    if index > super::MAX_STATIC_MATRIX_AXIS_DEPTH {
        return false;
    }
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
