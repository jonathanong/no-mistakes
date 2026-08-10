use super::fields::{
    bool_or_expression_field_valid, condition_field_valid, number_or_expression_field_valid,
    string_field_valid, STEP_CONDITION_CONTEXTS,
};
use super::values::{only_keys, scalar_mapping_valid};
use serde_yaml::{Mapping, Value};
use std::collections::BTreeSet;

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

pub(crate) fn steps_shape_valid(job: &Value) -> bool {
    let Some(steps) = job.get("steps") else {
        return job.get("uses").is_some();
    };
    steps.as_sequence().is_some_and(|steps| {
        !steps.is_empty()
            && steps
                .iter()
                .all(|step| step.as_mapping().is_some_and(step_shape_valid))
            && step_ids_valid(steps)
    })
}

fn step_ids_valid(steps: &[Value]) -> bool {
    let mut ids = BTreeSet::new();
    steps.iter().all(|step| {
        step.as_mapping()
            .and_then(|step| step.get("id"))
            .is_none_or(|id| {
                id.as_str()
                    .is_some_and(|id| valid_step_id(id) && ids.insert(id.to_ascii_lowercase()))
            })
    })
}

fn valid_step_id(id: &str) -> bool {
    let mut characters = id.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && characters
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
}

fn step_shape_valid(step: &Mapping) -> bool {
    match (step.get("run"), step.get("uses")) {
        (Some(Value::String(command)), None)
            if !command.is_empty()
                && super::super::super::super::expressions::interpolated_expression_valid(
                    command,
                ) =>
        {
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

fn shared_step_fields_valid(step: &Mapping) -> bool {
    string_field_valid(step, "name")
        && string_field_valid(step, "id")
        && condition_field_valid(step.get("if"), STEP_CONDITION_CONTEXTS, true)
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
        && super::super::valid_remote_reference(reference)
}
