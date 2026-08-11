use serde_yaml::Value;

use crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::{
    interpolated_expression_contexts_available, reduce_context_free_interpolations,
    typed_scalar_expression_contexts_available, ContextFreeInterpolation, StaticExpressionType,
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

fn valid_concurrency_group(value: &str, allowed_contexts: &[&str]) -> bool {
    !value.is_empty()
        && interpolated_expression_contexts_available(value, allowed_contexts)
        && match reduce_context_free_interpolations(value) {
            ContextFreeInterpolation::Static(value) => !value.is_empty(),
            ContextFreeInterpolation::Dynamic => true,
            ContextFreeInterpolation::Invalid => false,
        }
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
