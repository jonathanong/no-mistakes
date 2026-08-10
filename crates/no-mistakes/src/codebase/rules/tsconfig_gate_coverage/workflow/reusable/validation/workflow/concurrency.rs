use serde_yaml::Value;

use crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::{
    complete_expression_contexts_available, interpolated_expression_contexts_available,
};

pub(crate) fn job_concurrency_shape_valid(value: Option<&Value>) -> bool {
    const JOB_CONCURRENCY_CONTEXTS: &[&str] =
        &["github", "needs", "strategy", "matrix", "inputs", "vars"];
    value.is_none_or(|value| {
        value.as_str().is_some_and(|value| {
            !value.is_empty()
                && interpolated_expression_contexts_available(value, JOB_CONCURRENCY_CONTEXTS)
        }) || value.as_mapping().is_some_and(|concurrency| {
            concurrency.keys().all(|key| {
                key.as_str()
                    .is_some_and(|key| matches!(key, "group" | "cancel-in-progress"))
            }) && concurrency.get("group").is_some_and(|value| {
                value.as_str().is_some_and(|value| {
                    !value.is_empty()
                        && interpolated_expression_contexts_available(
                            value,
                            JOB_CONCURRENCY_CONTEXTS,
                        )
                })
            }) && concurrency.get("cancel-in-progress").is_none_or(|value| {
                value.is_bool()
                    || value.as_str().is_some_and(|value| {
                        complete_expression_contexts_available(value, JOB_CONCURRENCY_CONTEXTS)
                    })
            })
        })
    })
}

pub(crate) fn workflow_concurrency_shape_valid(value: Option<&Value>) -> bool {
    const WORKFLOW_CONCURRENCY_CONTEXTS: &[&str] = &["github", "inputs", "vars"];
    value.is_none_or(|value| {
        value.as_str().is_some_and(|value| {
            !value.is_empty()
                && interpolated_expression_contexts_available(value, WORKFLOW_CONCURRENCY_CONTEXTS)
        }) || value.as_mapping().is_some_and(|concurrency| {
            concurrency.keys().all(|key| {
                key.as_str()
                    .is_some_and(|key| matches!(key, "group" | "cancel-in-progress"))
            }) && concurrency.get("group").is_some_and(|value| {
                value.as_str().is_some_and(|value| {
                    !value.is_empty()
                        && interpolated_expression_contexts_available(
                            value,
                            WORKFLOW_CONCURRENCY_CONTEXTS,
                        )
                })
            }) && concurrency.get("cancel-in-progress").is_none_or(|value| {
                value.is_bool()
                    || value.as_str().is_some_and(|value| {
                        complete_expression_contexts_available(value, WORKFLOW_CONCURRENCY_CONTEXTS)
                    })
            })
        })
    })
}
