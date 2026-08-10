use serde_yaml::Value;
use std::collections::BTreeMap;

mod traversal;
use traversal::{count_unexcluded, has_applicable_combination};

const MATRIX_JOB_LIMIT: usize = 256;
const STATIC_MATRIX_ENUMERATION_LIMIT: usize = (MATRIX_JOB_LIMIT + 1) * 64;

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

enum StaticMappings<'a> {
    Static(Vec<&'a serde_yaml::Mapping>),
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
            .is_some_and(super::super::super::complete_expression);
    };
    match static_matrix_job_count(matrix) {
        StaticMatrixJobCount::Known(count) => count <= MATRIX_JOB_LIMIT,
        StaticMatrixJobCount::Dynamic => true,
        StaticMatrixJobCount::Invalid => false,
    }
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
            Value::Sequence(values)
                if values
                    .iter()
                    .all(|value| !matches!(value, Value::Sequence(_))) =>
            {
                axes.push((name.to_string(), values.clone()));
            }
            Value::String(expression) if super::super::super::complete_expression(expression) => {
                dynamic = true;
            }
            Value::Sequence(_) => dynamic = true,
            _ => return StaticMatrixAxes::Invalid,
        }
    }
    if dynamic {
        StaticMatrixAxes::Dynamic
    } else {
        StaticMatrixAxes::Static(axes)
    }
}

fn static_matrix_job_count(mapping: &serde_yaml::Mapping) -> StaticMatrixJobCount {
    let axes = match static_matrix_axes(mapping) {
        StaticMatrixAxes::Static(axes) => axes,
        StaticMatrixAxes::Dynamic => return StaticMatrixJobCount::Dynamic,
        StaticMatrixAxes::Invalid => return StaticMatrixJobCount::Invalid,
    };
    let exclusions = match static_mappings(mapping.get("exclude")) {
        StaticMappings::Static(mappings) => mappings,
        StaticMappings::Dynamic => return StaticMatrixJobCount::Dynamic,
        StaticMappings::Invalid => return StaticMatrixJobCount::Invalid,
    };
    let includes = match static_mappings(mapping.get("include")) {
        StaticMappings::Static(mappings) => mappings,
        StaticMappings::Dynamic => return StaticMatrixJobCount::Dynamic,
        StaticMappings::Invalid => return StaticMatrixJobCount::Invalid,
    };
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
            include,
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

fn static_mappings(value: Option<&Value>) -> StaticMappings<'_> {
    match value {
        Some(Value::Sequence(items)) => items
            .iter()
            .map(Value::as_mapping)
            .collect::<Option<_>>()
            .map_or(StaticMappings::Invalid, StaticMappings::Static),
        Some(Value::String(expression)) if super::super::super::complete_expression(expression) => {
            StaticMappings::Dynamic
        }
        Some(_) => StaticMappings::Invalid,
        None => StaticMappings::Static(Vec::new()),
    }
}

#[cfg(test)]
mod tests;
