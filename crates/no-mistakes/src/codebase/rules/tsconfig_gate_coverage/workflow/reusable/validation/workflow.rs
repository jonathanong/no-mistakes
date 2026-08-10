mod concurrency;

use serde_yaml::{Mapping, Value};

use super::super::super::expressions::{
    interpolated_expression_contexts_available, interpolated_expression_valid,
};

pub(super) use concurrency::{job_concurrency_shape_valid, workflow_concurrency_shape_valid};

pub(crate) fn workflow_shape_valid(workflow: &Value) -> bool {
    let Some(workflow) = workflow.as_mapping() else {
        return false;
    };
    only_workflow_keys(workflow)
        && string_field_valid(workflow, "name")
        && run_name_field_valid(workflow)
        && scalar_mapping_valid(workflow.get("env"))
        && permissions_shape_valid(workflow.get("permissions"))
        && workflow_defaults_shape_valid(workflow.get("defaults"))
        && workflow_concurrency_shape_valid(workflow.get("concurrency"))
}

fn only_workflow_keys(workflow: &Mapping) -> bool {
    workflow.keys().all(|key| {
        key.as_str().is_some_and(|key| {
            matches!(
                key,
                "name"
                    | "run-name"
                    | "on"
                    | "permissions"
                    | "env"
                    | "defaults"
                    | "concurrency"
                    | "jobs"
            )
        })
    })
}

fn string_field_valid(workflow: &Mapping, field: &str) -> bool {
    workflow.get(field).is_none_or(Value::is_string)
}

fn run_name_field_valid(workflow: &Mapping) -> bool {
    const RUN_NAME_CONTEXTS: &[&str] = &["github", "inputs", "vars"];
    workflow.get("run-name").is_none_or(|value| {
        value.as_str().is_some_and(|value| {
            interpolated_expression_contexts_available(value, RUN_NAME_CONTEXTS)
        })
    })
}

fn valid_nonempty_literal_string(value: &Value) -> bool {
    value.as_str().is_some_and(|value| {
        !value.is_empty() && !value.contains("${{") && interpolated_expression_valid(value)
    })
}

pub(super) fn scalar_mapping_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value.as_mapping().is_some_and(|mapping| {
            mapping.iter().all(|(name, value)| {
                name.is_string()
                    && match value {
                        Value::String(value) => interpolated_expression_valid(value),
                        Value::Bool(_) | Value::Number(_) => true,
                        _ => false,
                    }
            })
        })
    })
}

pub(super) fn permissions_shape_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value
            .as_str()
            .is_some_and(|value| matches!(value, "read-all" | "write-all"))
            || value.as_mapping().is_some_and(|mapping| {
                mapping.iter().all(|(name, value)| {
                    name.as_str().is_some_and(permission_scope_valid)
                        && value
                            .as_str()
                            .is_some_and(|value| permission_value_valid(name, value))
                })
            })
    })
}

fn permission_scope_valid(scope: &str) -> bool {
    matches!(
        scope,
        "actions"
            | "artifact-metadata"
            | "attestations"
            | "checks"
            | "contents"
            | "deployments"
            | "discussions"
            | "id-token"
            | "issues"
            | "models"
            | "packages"
            | "pages"
            | "pull-requests"
            | "repository-projects"
            | "security-events"
            | "statuses"
    )
}

fn permission_value_valid(scope: &Value, value: &str) -> bool {
    let Some(scope) = scope.as_str() else {
        return false;
    };
    match scope {
        "id-token" => matches!(value, "write" | "none"),
        "models" => matches!(value, "read" | "none"),
        _ => matches!(value, "read" | "write" | "none"),
    }
}

/// GitHub does not evaluate expressions in workflow-level `defaults.run`.
/// Job-level defaults have separate context rules and retain their own validator.
pub(super) fn workflow_defaults_shape_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value.as_mapping().is_some_and(|defaults| {
            defaults.len() == 1
                && defaults.get("run").is_some_and(|run| {
                    run.as_mapping().is_some_and(|run| {
                        !run.is_empty()
                            && run.keys().all(|key| {
                                key.as_str()
                                    .is_some_and(|key| matches!(key, "shell" | "working-directory"))
                            })
                            && run.values().all(valid_nonempty_literal_string)
                    })
                })
        })
    })
}

pub(super) fn job_defaults_shape_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value.as_mapping().is_some_and(|defaults| {
            defaults.len() == 1
                && defaults.get("run").is_some_and(|run| {
                    run.as_mapping().is_some_and(|run| {
                        !run.is_empty()
                            && run.keys().all(|key| {
                                key.as_str()
                                    .is_some_and(|key| matches!(key, "shell" | "working-directory"))
                            })
                            && run.values().all(valid_nonempty_literal_string)
                    })
                })
        })
    })
}

#[cfg(test)]
mod tests;
