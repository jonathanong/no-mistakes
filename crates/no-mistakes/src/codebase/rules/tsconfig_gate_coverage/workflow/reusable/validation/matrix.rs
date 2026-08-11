use serde_yaml::Value;
use std::collections::BTreeMap;

mod mappings;
use mappings::{static_mappings, StaticMappings};
mod traversal;
use traversal::{count_unexcluded, has_applicable_combination};
mod combinations;
pub(in super::super) use combinations::{static_matrix_combinations, MatrixCombinations};

const MATRIX_JOB_LIMIT: usize = 256;
// Recursive matrix traversal needs a finite axis-depth bound even when every
// axis has one value and the cartesian product remains one job.
const STATIC_MATRIX_AXIS_LIMIT: usize = MATRIX_JOB_LIMIT;
const STATIC_MATRIX_ENUMERATION_LIMIT: usize = (MATRIX_JOB_LIMIT + 1) * 64;
pub(super) const MAX_STATIC_MATRIX_AXIS_DEPTH: usize = MATRIX_JOB_LIMIT;

enum StaticMatrixAxes {
    Static(Vec<(String, Vec<Value>)>),
    Dynamic,
    Invalid,
}

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
    super::super::super::complete_expression_may_be_mapping(value)
}

fn static_matrix_axes(mapping: &serde_yaml::Mapping) -> StaticMatrixAxes {
    let mut axes = Vec::new();
    let mut dynamic = false;
    for (name, values) in mapping {
        if matches!(name.as_str(), Some("include" | "exclude")) {
            continue;
        }
        let Some(name) = name.as_str() else {
            return StaticMatrixAxes::Invalid;
        };
        match values {
            Value::Sequence(values) => match resolved_static_axis_values(values) {
                ResolvedStaticAxisValues::Static(values) if values.is_empty() => {
                    return StaticMatrixAxes::Invalid;
                }
                ResolvedStaticAxisValues::Static(values) => axes.push((name.to_string(), values)),
                ResolvedStaticAxisValues::Dynamic => dynamic = true,
                ResolvedStaticAxisValues::Invalid => return StaticMatrixAxes::Invalid,
            },
            Value::String(expression) if super::super::super::complete_expression(expression) => {
                dynamic = true;
            }
            _ => return StaticMatrixAxes::Invalid,
        }
    }
    if dynamic {
        StaticMatrixAxes::Dynamic
    } else if axes.len() > STATIC_MATRIX_AXIS_LIMIT {
        StaticMatrixAxes::Invalid
    } else {
        StaticMatrixAxes::Static(axes)
    }
}

/// Matrix axis entries are evaluated individually. Resolve context-free
/// expressions so the cartesian product and include/exclude matching use the
/// same typed values GitHub Actions sees at runtime.
enum ResolvedStaticAxisValues {
    Static(Vec<Value>),
    Dynamic,
    Invalid,
}

fn resolved_static_axis_values(values: &[Value]) -> ResolvedStaticAxisValues {
    let mut resolved = Vec::with_capacity(values.len());
    for value in values {
        match value {
            Value::Sequence(_) => return ResolvedStaticAxisValues::Dynamic,
            Value::String(expression)
                if expression.trim().starts_with("${{")
                    && !super::super::super::complete_expression(expression) =>
            {
                return ResolvedStaticAxisValues::Invalid;
            }
            Value::String(expression) if super::super::super::complete_expression(expression) => {
                let Some(value) =
                    super::super::super::complete_literal_expression_value(expression)
                else {
                    return ResolvedStaticAxisValues::Dynamic;
                };
                if matches!(value, Value::Sequence(_)) {
                    return ResolvedStaticAxisValues::Dynamic;
                }
                resolved.push(value);
            }
            value => resolved.push(value.clone()),
        }
    }
    ResolvedStaticAxisValues::Static(resolved)
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
