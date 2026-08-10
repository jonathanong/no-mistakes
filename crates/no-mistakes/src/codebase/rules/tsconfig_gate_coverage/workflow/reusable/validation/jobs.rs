use serde_yaml::{Mapping, Value};
use std::collections::BTreeSet;

pub(crate) fn steps_shape_valid(job: &Value) -> bool {
    let Some(steps) = job.get("steps") else {
        return job.get("uses").is_some();
    };
    steps.as_sequence().is_some_and(|steps| {
        !steps.is_empty()
            && steps
                .iter()
                .all(|step| step.as_mapping().is_some_and(step_shape_valid))
    })
}

fn step_shape_valid(step: &Mapping) -> bool {
    match (step.get("run"), step.get("uses")) {
        (Some(Value::String(command)), None) if !command.is_empty() => {
            only_keys(step, RUN_STEP_KEYS) && shared_step_fields_valid(step)
        }
        (None, Some(Value::String(target))) if action_target_valid(target) => {
            only_keys(step, ACTION_STEP_KEYS)
                && shared_step_fields_valid(step)
                && scalar_mapping_valid(step.get("with"))
        }
        _ => false,
    }
}

const RUN_STEP_KEYS: &[&str] = &[
    "name",
    "id",
    "if",
    "run",
    "working-directory",
    "shell",
    "env",
    "continue-on-error",
    "timeout-minutes",
];

const ACTION_STEP_KEYS: &[&str] = &[
    "name",
    "id",
    "if",
    "uses",
    "with",
    "env",
    "continue-on-error",
    "timeout-minutes",
];

fn shared_step_fields_valid(step: &Mapping) -> bool {
    string_field_valid(step, "name")
        && string_field_valid(step, "id")
        && condition_field_valid(step.get("if"))
        && string_field_valid(step, "working-directory")
        && string_field_valid(step, "shell")
        && scalar_mapping_valid(step.get("env"))
        && bool_or_expression_field_valid(step, "continue-on-error")
        && number_or_expression_field_valid(step, "timeout-minutes")
}

fn action_target_valid(target: &str) -> bool {
    if target.contains("${{") || target.chars().any(char::is_whitespace) {
        return false;
    }
    if let Some(path) = target.strip_prefix("./") {
        return !path.is_empty()
            && !path.contains('\\')
            && path
                .split('/')
                .all(|segment| !matches!(segment, "" | "." | ".."));
    }
    if let Some(image) = target.strip_prefix("docker://") {
        return !image.is_empty();
    }
    let Some((path, reference)) = target.rsplit_once('@') else {
        return false;
    };
    let mut segments = path.split('/');
    segments.next().is_some_and(|owner| !owner.is_empty())
        && segments
            .next()
            .is_some_and(|repository| !repository.is_empty())
        && segments.all(|segment| !segment.is_empty())
        && !reference.is_empty()
}

pub(crate) fn call_bindings_shape_valid(job: &Value) -> bool {
    binding_mapping_valid(job.get("with"))
        && match job.get("secrets") {
            Some(Value::String(value)) => value == "inherit",
            value => binding_mapping_valid(value),
        }
}

fn binding_mapping_valid(value: Option<&Value>) -> bool {
    let Some(value) = value else {
        return true;
    };
    let Some(mapping) = value.as_mapping() else {
        return false;
    };
    unique_scalar_bindings(mapping)
}

fn scalar_mapping_valid(value: Option<&Value>) -> bool {
    let Some(value) = value else {
        return true;
    };
    value.as_mapping().is_some_and(|mapping| {
        mapping.iter().all(|(name, value)| {
            name.is_string()
                && matches!(value, Value::Bool(_) | Value::Number(_) | Value::String(_))
        })
    })
}

fn only_keys(mapping: &Mapping, allowed: &[&str]) -> bool {
    mapping
        .keys()
        .all(|key| key.as_str().is_some_and(|key| allowed.contains(&key)))
}

fn string_field_valid(mapping: &Mapping, field: &str) -> bool {
    mapping.get(field).is_none() || mapping.get(field).is_some_and(Value::is_string)
}

pub(super) fn condition_field_valid(value: Option<&Value>) -> bool {
    value.is_none()
        || value.is_some_and(|value| {
            value.is_bool()
                || value
                    .as_str()
                    .is_some_and(super::super::super::expressions::condition_expression_valid)
        })
}

fn bool_or_expression_field_valid(mapping: &Mapping, field: &str) -> bool {
    mapping.get(field).is_none()
        || mapping.get(field).is_some_and(|value| {
            value.as_bool().is_some()
                || value
                    .as_str()
                    .is_some_and(super::super::super::complete_expression)
        })
}

fn number_or_expression_field_valid(mapping: &Mapping, field: &str) -> bool {
    mapping.get(field).is_none()
        || mapping.get(field).is_some_and(|value| {
            value
                .as_u64()
                .is_some_and(|minutes| (1..=360).contains(&minutes))
                || value
                    .as_str()
                    .is_some_and(super::super::super::complete_expression)
        })
}

fn unique_scalar_bindings(mapping: &Mapping) -> bool {
    let mut names = BTreeSet::new();
    mapping.iter().all(|(name, value)| {
        name.as_str()
            .is_some_and(|name| names.insert(name.to_ascii_lowercase()))
            && matches!(value, Value::Bool(_) | Value::Number(_) | Value::String(_))
    })
}

#[cfg(test)]
mod tests;
