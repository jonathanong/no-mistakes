use serde_yaml::{Mapping, Value};

use super::super::super::super::expressions::{
    complete_expression_contexts_with_hash_files_available, complete_expression_type,
    complete_literal_expression_value, condition_expression_contexts_available,
    interpolated_expression_contexts_and_hash_files_available,
    interpolated_expression_contexts_available, interpolated_expression_valid,
    invalid_literal_from_json, StaticExpressionType,
};

mod strategy;
pub(super) use strategy::strategy_shape_valid;
pub(crate) use strategy::{fail_fast_enabled_for_inputs, strategy_configuration_valid_for_inputs};

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
pub(super) const JOB_TIMEOUT_CONTEXTS: &[&str] = JOB_CONTINUE_ON_ERROR_CONTEXTS;
pub(super) const STEP_TIMEOUT_CONTEXTS: &[&str] = STEP_CONTINUE_ON_ERROR_CONTEXTS;
pub(super) const JOB_NAME_CONTEXTS: &[&str] =
    &["github", "needs", "strategy", "matrix", "vars", "inputs"];
pub(super) const STEP_STRING_CONTEXTS: &[&str] = &[
    "github", "needs", "strategy", "matrix", "job", "runner", "env", "vars", "secrets", "steps",
    "inputs",
];

pub(super) fn string_field_valid(
    mapping: &Mapping,
    field: &str,
    allowed_contexts: &[&str],
    hash_files_available: bool,
) -> bool {
    mapping.get(field).is_none_or(|value| {
        value.as_str().is_some_and(|value| {
            if hash_files_available {
                interpolated_expression_contexts_and_hash_files_available(value, allowed_contexts)
            } else {
                interpolated_expression_contexts_available(value, allowed_contexts)
            }
        })
    })
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
                ) && matches!(
                    complete_expression_type(value),
                    Some(StaticExpressionType::Boolean | StaticExpressionType::Dynamic)
                )
            })
    })
}

pub(super) fn timeout_minutes_field_valid(
    mapping: &Mapping,
    field: &str,
    allowed_contexts: &[&str],
    hash_files_available: bool,
    maximum: Option<u64>,
) -> bool {
    mapping.get(field).is_none_or(|value| {
        value
            .as_u64()
            .is_some_and(|minutes| valid_timeout(minutes, maximum))
            || value.as_str().is_some_and(|value| {
                complete_expression_contexts_with_hash_files_available(
                    value,
                    allowed_contexts,
                    hash_files_available,
                ) && !invalid_literal_from_json(value)
                    && matches!(
                        complete_expression_type(value),
                        Some(StaticExpressionType::Number | StaticExpressionType::Dynamic)
                    )
                    && match complete_literal_expression_value(value) {
                        Some(literal) => literal
                            .as_u64()
                            .is_some_and(|minutes| valid_timeout(minutes, maximum)),
                        None => !complete_expression_contexts_with_hash_files_available(
                            value,
                            &[],
                            false,
                        ),
                    }
            })
    })
}

fn valid_timeout(minutes: u64, maximum: Option<u64>) -> bool {
    minutes > 0 && maximum.is_none_or(|maximum| minutes <= maximum)
}

pub(super) fn scalar_value_valid(value: &Value) -> bool {
    matches!(value, Value::Bool(_) | Value::Number(_))
        || value.as_str().is_some_and(interpolated_expression_valid)
}

#[cfg(test)]
mod tests;
