use serde_yaml::Value;

use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    complete_expression_static_string_value, complete_expression_static_value,
    resolve_static_interpolations, EnvironmentState, InputState, StaticValue,
};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::{
    interpolated_expression_contexts_available, interpolation_expressions_all,
    reduce_context_free_interpolations, typed_scalar_expression_contexts_available,
    ContextFreeInterpolation, StaticExpressionType,
};

pub(crate) fn job_concurrency_shape_valid(value: Option<&Value>) -> bool {
    const JOB_CONCURRENCY_CONTEXTS: &[&str] =
        &["github", "needs", "strategy", "matrix", "inputs", "vars"];
    value.is_none_or(|value| {
        value
            .as_str()
            .is_some_and(|value| valid_concurrency_group(value, JOB_CONCURRENCY_CONTEXTS))
            || value.as_mapping().is_some_and(|concurrency| {
                concurrency.keys().all(|key| {
                    key.as_str()
                        .is_some_and(|key| matches!(key, "group" | "cancel-in-progress"))
                }) && concurrency.get("group").is_some_and(|value| {
                    value.as_str().is_some_and(|value| {
                        valid_concurrency_group(value, JOB_CONCURRENCY_CONTEXTS)
                    })
                }) && concurrency
                    .get("cancel-in-progress")
                    .is_none_or(|value| cancel_in_progress_valid(value, JOB_CONCURRENCY_CONTEXTS))
            })
    })
}

pub(crate) fn workflow_concurrency_shape_valid(value: Option<&Value>) -> bool {
    const WORKFLOW_CONCURRENCY_CONTEXTS: &[&str] = &["github", "inputs", "vars"];
    value.is_none_or(|value| {
        value
            .as_str()
            .is_some_and(|value| valid_concurrency_group(value, WORKFLOW_CONCURRENCY_CONTEXTS))
            || value.as_mapping().is_some_and(|concurrency| {
                concurrency.keys().all(|key| {
                    key.as_str()
                        .is_some_and(|key| matches!(key, "group" | "cancel-in-progress"))
                }) && concurrency.get("group").is_some_and(|value| {
                    value.as_str().is_some_and(|value| {
                        valid_concurrency_group(value, WORKFLOW_CONCURRENCY_CONTEXTS)
                    })
                }) && concurrency.get("cancel-in-progress").is_none_or(|value| {
                    cancel_in_progress_valid(value, WORKFLOW_CONCURRENCY_CONTEXTS)
                })
            })
    })
}

pub(crate) fn job_concurrency_valid_for_inputs(value: Option<&Value>, inputs: &InputState) -> bool {
    concurrency_valid_for_inputs(value, inputs)
}

pub(crate) fn workflow_concurrency_valid_for_inputs(
    value: Option<&Value>,
    inputs: &InputState,
) -> bool {
    concurrency_valid_for_inputs(value, inputs)
}

fn valid_concurrency_group(value: &str, allowed_contexts: &[&str]) -> bool {
    !value.is_empty()
        && interpolated_expression_contexts_available(value, allowed_contexts)
        && match reduce_context_free_interpolations(value) {
            ContextFreeInterpolation::Static(value) => !value.is_empty(),
            ContextFreeInterpolation::Dynamic => true,
            ContextFreeInterpolation::Invalid => false,
        }
}

fn concurrency_valid_for_inputs(value: Option<&Value>, inputs: &InputState) -> bool {
    let Some(group) = value.and_then(concurrency_group) else {
        return value.is_none();
    };
    if !interpolation_expressions_all(group, |expression| {
        let expression = format!("${{{{ {expression} }}}}");
        complete_expression_static_value(&expression, inputs).is_none_or(|value| {
            !matches!(
                value,
                StaticValue::Sequence(_) | StaticValue::Mapping | StaticValue::NonStringable
            )
        })
    }) {
        return false;
    }
    let group_valid = if let Some(value) = complete_expression_static_string_value(group, inputs) {
        match value {
            StaticValue::String(value) => !value.trim().is_empty(),
            StaticValue::Unknown => resolved_concurrency_group_valid(group, inputs),
            StaticValue::Bool(_)
            | StaticValue::Number(_)
            | StaticValue::Null
            | StaticValue::Sequence(_)
            | StaticValue::Mapping
            | StaticValue::NonStringable
            | StaticValue::ExpressionError => false,
        }
    } else {
        resolved_concurrency_group_valid(group, inputs)
    };
    group_valid
        && concurrency_cancel_in_progress_valid_for_inputs(
            value.and_then(concurrency_cancel_in_progress),
            inputs,
        )
}

fn resolved_concurrency_group_valid(group: &str, inputs: &InputState) -> bool {
    resolve_static_interpolations(group, inputs, &EnvironmentState::default())
        .is_none_or(|group| !group.trim().is_empty())
}

fn concurrency_group(value: &Value) -> Option<&str> {
    value.as_str().or_else(|| {
        value
            .as_mapping()
            .and_then(|concurrency| concurrency.get("group"))
            .and_then(Value::as_str)
    })
}

fn concurrency_cancel_in_progress(value: &Value) -> Option<&Value> {
    value
        .as_mapping()
        .and_then(|concurrency| concurrency.get("cancel-in-progress"))
}

fn concurrency_cancel_in_progress_valid_for_inputs(
    value: Option<&Value>,
    inputs: &InputState,
) -> bool {
    value.is_none_or(|value| {
        value.is_bool()
            || value.as_str().is_some_and(|value| {
                complete_expression_static_value(value, inputs).is_none_or(|value| {
                    matches!(value, StaticValue::Bool(_) | StaticValue::Unknown)
                })
            })
    })
}

fn cancel_in_progress_valid(value: &Value, allowed_contexts: &[&str]) -> bool {
    value.is_bool()
        || value.as_str().is_some_and(|value| {
            typed_scalar_expression_contexts_available(
                value,
                allowed_contexts,
                StaticExpressionType::Boolean,
            )
        })
}
