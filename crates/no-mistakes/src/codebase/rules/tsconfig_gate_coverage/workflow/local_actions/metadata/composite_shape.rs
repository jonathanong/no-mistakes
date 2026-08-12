use super::shape::only_keys;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::{
    complete_expression_contexts_available, complete_expression_type,
    condition_expression_contexts_available, interpolated_expression_contexts_available,
    StaticExpressionType,
};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::reusable::validation::action_target_valid;
use serde_yaml::{Mapping, Value};

const COMPOSITE_STEP_CONTEXTS: &[&str] = &["github", "inputs", "steps", "runner", "env", "vars"];
const COMPOSITE_STEP_KEYS: &[&str] = &[
    "continue-on-error",
    "env",
    "id",
    "if",
    "name",
    "run",
    "shell",
    "uses",
    "with",
    "working-directory",
];

pub(super) fn steps_valid(steps: &[Value]) -> bool {
    !steps.is_empty()
        && steps
            .iter()
            .all(|step| step.as_mapping().is_some_and(step_valid))
}

pub(super) fn step_valid(step: &Mapping) -> bool {
    only_keys(step, COMPOSITE_STEP_KEYS)
        && shared_fields_valid(step)
        && match (step.get("run"), step.get("uses")) {
            (Some(Value::String(run)), None) => {
                !run.is_empty()
                    && interpolated_expression_contexts_available(run, COMPOSITE_STEP_CONTEXTS)
            }
            (None, Some(Value::String(uses))) => {
                !uses.is_empty()
                    && action_target_valid(uses)
                    && action_inputs_valid(step.get("with"))
            }
            _ => false,
        }
}

fn shared_fields_valid(step: &Mapping) -> bool {
    ["name", "id", "shell", "working-directory"]
        .into_iter()
        .all(|field| interpolated_field_valid(step.get(field)))
        && step.get("if").is_none_or(|value| {
            value.is_bool()
                || value.as_str().is_some_and(|value| {
                    condition_expression_contexts_available(value, COMPOSITE_STEP_CONTEXTS, false)
                })
        })
        && scalar_mapping_valid(step.get("env"))
        && step.get("continue-on-error").is_none_or(|value| {
            value.is_bool()
                || value.as_str().is_some_and(|value| {
                    complete_expression_contexts_available(value, COMPOSITE_STEP_CONTEXTS)
                        && matches!(
                            complete_expression_type(value),
                            Some(StaticExpressionType::Boolean | StaticExpressionType::Dynamic)
                        )
                })
        })
}

fn interpolated_field_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value.as_str().is_some_and(|value| {
            interpolated_expression_contexts_available(value, COMPOSITE_STEP_CONTEXTS)
        })
    })
}

fn scalar_mapping_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value.as_mapping().is_some_and(|mapping| {
            mapping.iter().all(|(name, value)| {
                name.is_string()
                    && (matches!(value, Value::Bool(_) | Value::Number(_))
                        || value.as_str().is_some_and(|value| {
                            interpolated_expression_contexts_available(
                                value,
                                COMPOSITE_STEP_CONTEXTS,
                            )
                        }))
            })
        })
    })
}

fn action_inputs_valid(value: Option<&Value>) -> bool {
    scalar_mapping_valid(value)
}
