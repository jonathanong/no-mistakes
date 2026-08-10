use super::values::only_keys;
use serde_yaml::{Mapping, Value};

use super::super::super::super::expressions::{
    complete_expression_contexts_available, complete_expression_contexts_with_hash_files_available,
    complete_expression_type, complete_literal_expression_value,
    condition_expression_contexts_available, interpolated_expression_valid, StaticExpressionType,
};

pub(super) const JOB_CONDITION_CONTEXTS: &[&str] = &["github", "needs", "vars", "inputs"];
pub(super) const STEP_CONDITION_CONTEXTS: &[&str] = &[
    "github", "needs", "strategy", "matrix", "job", "runner", "env", "vars", "steps", "inputs",
];
pub(super) const JOB_CONTINUE_ON_ERROR_CONTEXTS: &[&str] =
    &["github", "needs", "strategy", "matrix", "vars", "inputs"];
pub(super) const STEP_CONTINUE_ON_ERROR_CONTEXTS: &[&str] = &[
    "github", "needs", "strategy", "matrix", "job", "runner", "env", "vars", "secrets", "steps",
    "inputs",
];
const STRATEGY_CONTEXTS: &[&str] = &["github", "needs", "vars", "inputs"];

pub(super) fn strategy_shape_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value.as_mapping().is_some_and(|strategy| {
            !strategy.is_empty()
                && only_keys(strategy, &["fail-fast", "max-parallel", "matrix"])
                && strategy.get("fail-fast").is_none_or(|value| {
                    value.is_bool()
                        || value
                            .as_str()
                            .is_some_and(strategy_fail_fast_expression_valid)
                })
                && strategy.get("max-parallel").is_none_or(|value| {
                    value.as_u64().is_some_and(|value| value > 0)
                        || value
                            .as_str()
                            .is_some_and(strategy_max_parallel_expression_valid)
                })
        })
    })
}

fn strategy_fail_fast_expression_valid(value: &str) -> bool {
    complete_expression_contexts_available(value, STRATEGY_CONTEXTS)
        && matches!(
            complete_expression_type(value),
            Some(StaticExpressionType::Boolean | StaticExpressionType::Dynamic)
        )
}

fn strategy_max_parallel_expression_valid(value: &str) -> bool {
    complete_expression_contexts_available(value, STRATEGY_CONTEXTS)
        && matches!(
            complete_expression_type(value),
            Some(StaticExpressionType::Number | StaticExpressionType::Dynamic)
        )
        && complete_literal_expression_value(value)
            .is_none_or(|literal| literal.as_u64().is_some_and(|parallelism| parallelism > 0))
}

pub(super) fn string_field_valid(mapping: &Mapping, field: &str) -> bool {
    mapping
        .get(field)
        .is_none_or(|value| value.as_str().is_some_and(interpolated_expression_valid))
}

pub(crate) fn condition_field_valid(
    value: Option<&Value>,
    allowed_contexts: &[&str],
    hash_files_available: bool,
) -> bool {
    value.is_none_or(|value| {
        value.is_bool()
            || value.as_str().is_some_and(|value| {
                condition_expression_contexts_available(
                    value,
                    allowed_contexts,
                    hash_files_available,
                )
            })
    })
}

pub(super) fn bool_or_expression_field_valid(
    mapping: &Mapping,
    field: &str,
    allowed_contexts: &[&str],
    hash_files_available: bool,
) -> bool {
    mapping.get(field).is_none_or(|value| {
        value.is_bool()
            || value.as_str().is_some_and(|value| {
                complete_expression_contexts_with_hash_files_available(
                    value,
                    allowed_contexts,
                    hash_files_available,
                )
            })
    })
}

pub(super) fn number_or_expression_field_valid(mapping: &Mapping, field: &str) -> bool {
    mapping.get(field).is_none_or(|value| {
        value
            .as_u64()
            .is_some_and(|minutes| (1..=360).contains(&minutes))
            || value
                .as_str()
                .is_some_and(super::super::super::super::complete_expression)
    })
}

pub(super) fn scalar_value_valid(value: &Value) -> bool {
    matches!(value, Value::Bool(_) | Value::Number(_))
        || value.as_str().is_some_and(interpolated_expression_valid)
}

#[cfg(test)]
mod tests;
