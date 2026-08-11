use serde_yaml::{Mapping, Value};

use super::super::super::super::conditions::{
    complete_expression_static_string_value, complete_expression_static_value,
    resolve_static_interpolations, EnvironmentState, InputState, StaticValue,
};
use super::super::super::super::expressions::{
    interpolated_expression_contexts_and_hash_files_available,
    interpolated_expression_contexts_available, interpolated_expression_valid,
    interpolation_expressions_all,
};
const RUNS_ON_CONTEXTS: &[&str] = &["github", "inputs", "vars", "needs", "strategy", "matrix"];
pub(super) const JOB_ENV_CONTEXTS: &[&str] = &[
    "github", "needs", "strategy", "matrix", "vars", "secrets", "inputs",
];
pub(super) const STEP_ENV_CONTEXTS: &[&str] = &[
    "github", "needs", "strategy", "matrix", "job", "runner", "env", "vars", "secrets", "steps",
    "inputs",
];
const JOB_OUTPUT_CONTEXTS: &[&str] = &[
    "github", "needs", "strategy", "matrix", "job", "runner", "env", "vars", "secrets", "steps",
    "inputs",
];
const ENVIRONMENT_NAME_CONTEXTS: &[&str] =
    &["github", "inputs", "vars", "needs", "strategy", "matrix"];
const ENVIRONMENT_URL_CONTEXTS: &[&str] = &[
    "github", "inputs", "vars", "needs", "strategy", "matrix", "job", "runner", "env", "steps",
];
pub(super) fn scalar_mapping_valid(
    value: Option<&Value>,
    allowed_contexts: &[&str],
    hash_files_available: bool,
) -> bool {
    value.is_none_or(|value| {
        value.as_mapping().is_some_and(|mapping| {
            mapping.iter().all(|(name, value)| {
                name.is_string()
                    && scalar_value_valid(value, allowed_contexts, hash_files_available)
            })
        })
    })
}

pub(super) fn only_keys(mapping: &Mapping, allowed: &[&str]) -> bool {
    mapping
        .keys()
        .all(|key| key.as_str().is_some_and(|key| allowed.contains(&key)))
}

pub(super) fn runs_on_shape_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value.as_str().is_some_and(valid_runs_on_label)
            || value
                .as_sequence()
                .is_some_and(|labels| valid_runs_on_labels(labels))
            || value.as_mapping().is_some_and(|selection| {
                !selection.is_empty()
                    && only_keys(selection, &["group", "labels"])
                    && selection
                        .get("group")
                        .is_none_or(|group| group.as_str().is_some_and(valid_runs_on_label))
                    && selection.get("labels").is_none_or(|labels| {
                        labels.as_str().is_some_and(valid_runs_on_label)
                            || labels
                                .as_sequence()
                                .is_some_and(|labels| valid_runs_on_labels(labels))
                    })
            })
    })
}

fn valid_runs_on_labels(labels: &[Value]) -> bool {
    !labels.is_empty()
        && labels
            .iter()
            .all(|label| label.as_str().is_some_and(valid_runs_on_label))
}

fn valid_runs_on_label(value: &str) -> bool {
    !value.is_empty() && interpolated_expression_contexts_available(value, RUNS_ON_CONTEXTS)
}

pub(super) fn environment_shape_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value.as_str().is_some_and(valid_environment_name)
            || value.as_mapping().is_some_and(|environment| {
                only_keys(environment, &["name", "url"])
                    && environment
                        .get("name")
                        .and_then(Value::as_str)
                        .is_some_and(valid_environment_name)
                    && environment
                        .get("url")
                        .is_none_or(|url| url.as_str().is_some_and(valid_environment_url))
            })
    })
}

pub(crate) fn environment_configuration_valid_for_inputs(job: &Value, inputs: &InputState) -> bool {
    let Some(environment) = job.get("environment") else {
        return true;
    };
    let name = environment.as_str().or_else(|| {
        environment
            .as_mapping()
            .and_then(|environment| environment.get("name"))
            .and_then(Value::as_str)
    });
    name.is_some_and(|name| {
        if let Some(value) = complete_expression_static_string_value(name, inputs) {
            if !matches!(value, StaticValue::Unknown) {
                return value
                    .function_string()
                    .is_some_and(|name| !name.trim().is_empty());
            }
        }
        resolve_static_interpolations(name, inputs, &EnvironmentState::default())
            .is_none_or(|name| !name.trim().is_empty())
    }) && environment_url_valid_for_inputs(environment, inputs)
}

fn environment_url_valid_for_inputs(environment: &Value, inputs: &InputState) -> bool {
    let Some(url) = environment
        .as_mapping()
        .and_then(|environment| environment.get("url"))
    else {
        return true;
    };
    let Some(url) = url.as_str() else {
        return false;
    };
    interpolation_expressions_all(url, |expression| {
        let expression = format!("${{{{ {expression} }}}}");
        complete_expression_static_value(&expression, inputs).is_none_or(|value| {
            matches!(value, StaticValue::Unknown) || value.function_string().is_some()
        })
    })
}

pub(super) fn outputs_shape_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value.as_mapping().is_some_and(|outputs| {
            !outputs.is_empty()
                && outputs.iter().all(|(name, expression)| {
                    name.as_str().is_some_and(|name| !name.is_empty())
                        && expression.as_str().is_some_and(valid_job_output_expression)
                })
        })
    })
}

fn valid_nonempty_interpolated_string(value: &str) -> bool {
    !value.is_empty() && interpolated_expression_valid(value)
}

fn scalar_value_valid(
    value: &Value,
    allowed_contexts: &[&str],
    hash_files_available: bool,
) -> bool {
    matches!(value, Value::Bool(_) | Value::Number(_))
        || value.as_str().is_some_and(|value| {
            if hash_files_available {
                interpolated_expression_contexts_and_hash_files_available(value, allowed_contexts)
            } else {
                interpolated_expression_contexts_available(value, allowed_contexts)
            }
        })
}

fn valid_job_output_expression(value: &str) -> bool {
    !value.is_empty() && interpolated_expression_contexts_available(value, JOB_OUTPUT_CONTEXTS)
}

fn valid_environment_name(value: &str) -> bool {
    valid_nonempty_interpolated_string(value)
        && interpolated_expression_contexts_available(value, ENVIRONMENT_NAME_CONTEXTS)
}

fn valid_environment_url(value: &str) -> bool {
    valid_nonempty_interpolated_string(value)
        && interpolated_expression_contexts_available(value, ENVIRONMENT_URL_CONTEXTS)
}
