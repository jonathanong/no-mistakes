use crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::{
    complete_expression_contexts_available, interpolated_expression_contexts_available,
};
use serde_yaml::Value;
use std::collections::BTreeMap;

mod axes;
use axes::{static_matrix_axes, StaticMatrixAxes};
mod mappings;
use mappings::{static_mappings, StaticMappings};
mod traversal;
use traversal::{count_unexcluded, has_applicable_combination};
mod combinations;
pub(in super::super) use combinations::{static_matrix_combinations, MatrixCombinations};

const MATRIX_JOB_LIMIT: usize = 256;
const MATRIX_CONTEXTS: &[&str] = &["github", "needs", "vars", "inputs"];
// Recursive matrix traversal needs a finite axis-depth bound even when every
// axis has one value and the cartesian product remains one job.
const STATIC_MATRIX_AXIS_LIMIT: usize = MATRIX_JOB_LIMIT;
const STATIC_MATRIX_ENUMERATION_LIMIT: usize = (MATRIX_JOB_LIMIT + 1) * 64;
pub(super) const MAX_STATIC_MATRIX_AXIS_DEPTH: usize = MATRIX_JOB_LIMIT;

enum StaticMatrixJobCount {
    Known(usize),
    Dynamic,
    Invalid,
}

pub(crate) fn zero_instance_matrix(job: &Value) -> bool {
    let Some(matrix) = job
        .get("strategy")
        .and_then(|strategy| strategy.get("matrix"))
        .and_then(Value::as_mapping)
    else {
        return false;
    };
    matches!(
        static_matrix_job_count(matrix),
        StaticMatrixJobCount::Known(0)
    )
}

pub(crate) fn matrix_shape_valid(job: &Value) -> bool {
    let Some(strategy) = job.get("strategy") else {
        return true;
    };
    let Some(strategy) = strategy.as_mapping() else {
        return false;
    };
    let Some(matrix) = strategy.get("matrix") else {
        return true;
    };
    let Some(matrix) = matrix.as_mapping() else {
        return matrix
            .as_str()
            .is_some_and(matrix_expression_may_be_mapping);
    };
    match static_matrix_job_count(matrix) {
        StaticMatrixJobCount::Known(count) => count <= MATRIX_JOB_LIMIT,
        StaticMatrixJobCount::Dynamic => true,
        StaticMatrixJobCount::Invalid => false,
    }
}

fn matrix_expression_may_be_mapping(value: &str) -> bool {
    matrix_expression_valid(value) && super::super::super::complete_expression_may_be_mapping(value)
}

fn matrix_expression_valid(value: &str) -> bool {
    complete_expression_contexts_available(value, MATRIX_CONTEXTS)
}

fn matrix_interpolated_expression_valid(value: &str) -> bool {
    interpolated_expression_contexts_available(value, MATRIX_CONTEXTS)
}

fn static_matrix_job_count(mapping: &serde_yaml::Mapping) -> StaticMatrixJobCount {
    let axes = match static_matrix_axes(mapping) {
        axes @ (StaticMatrixAxes::Static(_) | StaticMatrixAxes::Dynamic) => axes,
        StaticMatrixAxes::Invalid => return StaticMatrixJobCount::Invalid,
    };
    let exclusions = match static_mappings(mapping.get("exclude")) {
        mappings @ (StaticMappings::Static(_) | StaticMappings::Dynamic) => mappings,
        StaticMappings::Invalid => return StaticMatrixJobCount::Invalid,
    };
    let includes = match static_mappings(mapping.get("include")) {
        mappings @ (StaticMappings::Static(_) | StaticMappings::Dynamic) => mappings,
        StaticMappings::Invalid => return StaticMatrixJobCount::Invalid,
    };
    let (axes, exclusions, includes) = match (axes, exclusions, includes) {
        (StaticMatrixAxes::Dynamic, _, _)
        | (_, StaticMappings::Dynamic, _)
        | (_, _, StaticMappings::Dynamic) => {
            return StaticMatrixJobCount::Dynamic;
        }
        (
            StaticMatrixAxes::Static(axes),
            StaticMappings::Static(exclusions),
            StaticMappings::Static(includes),
        ) => (axes, exclusions, includes),
        (StaticMatrixAxes::Invalid, _, _)
        | (_, StaticMappings::Invalid, _)
        | (_, _, StaticMappings::Invalid) => {
            return StaticMatrixJobCount::Invalid;
        }
    };
    if axes.is_empty() && includes.is_empty() {
        return StaticMatrixJobCount::Invalid;
    }
    let mut values = BTreeMap::new();
    let mut states_remaining = STATIC_MATRIX_ENUMERATION_LIMIT;
    let Some(mut count) = count_unexcluded(
        &axes,
        &exclusions,
        0,
        &mut values,
        MATRIX_JOB_LIMIT + 1,
        &mut states_remaining,
    ) else {
        return StaticMatrixJobCount::Invalid;
    };
    for include in includes {
        if count > MATRIX_JOB_LIMIT {
            break;
        }
        values.clear();
        let Some(applicable) = has_applicable_combination(
            &axes,
            &exclusions,
            &include,
            0,
            &mut values,
            &mut states_remaining,
        ) else {
            return StaticMatrixJobCount::Invalid;
        };
        if !applicable {
            count = count.saturating_add(1).min(MATRIX_JOB_LIMIT + 1);
        }
    }
    StaticMatrixJobCount::Known(count)
}

#[cfg(test)]
mod tests;
