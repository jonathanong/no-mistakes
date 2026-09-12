use super::super::values::only_keys;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    complete_expression_static_value, resolve_static_interpolations, EnvironmentState, InputState,
    StaticValue,
};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::{
    complete_expression_contexts_available, complete_expression_type,
    complete_literal_expression_value, invalid_literal_from_json, StaticExpressionType,
};
use serde_yaml::Value;

const STRATEGY_CONTEXTS: &[&str] = &["github", "needs", "vars", "inputs"];

pub(in super::super) fn strategy_shape_valid(value: Option<&Value>) -> bool {
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

pub(crate) fn strategy_configuration_valid_for_inputs(job: &Value, inputs: &InputState) -> bool {
    let Some(strategy) = job.get("strategy").and_then(Value::as_mapping) else {
        return true;
    };
    strategy
        .get("fail-fast")
        .is_none_or(|value| fail_fast_valid_for_inputs(value, inputs))
        && strategy
            .get("max-parallel")
            .is_none_or(|value| max_parallel_valid_for_inputs(value, inputs))
}

/// GitHub Actions enables matrix fail-fast unless a statically resolved value
/// explicitly disables it. Dynamic values cannot prove that queued siblings
/// were cancelled, so callers must continue scanning them.
pub(crate) fn strategy_fail_fast_enabled_for_inputs(job: &Value, inputs: &InputState) -> bool {
    matches!(
        strategy_context_values_for_inputs(job, inputs, None).0,
        StaticValue::Bool(true)
    )
}

pub(crate) fn strategy_context_values_for_inputs(
    job: &Value,
    inputs: &InputState,
    job_total: Option<usize>,
) -> (StaticValue, StaticValue) {
    let strategy = job.get("strategy").and_then(Value::as_mapping);
    let fail_fast = strategy
        .and_then(|strategy| strategy.get("fail-fast"))
        .map_or(StaticValue::Bool(true), |value| match value {
            Value::Bool(enabled) => StaticValue::Bool(*enabled),
            Value::String(expression) => complete_expression_static_value(expression, inputs)
                .filter(|value| matches!(value, StaticValue::Bool(_) | StaticValue::Unknown))
                .unwrap_or(StaticValue::Unknown),
            _ => StaticValue::Unknown,
        });
    let max_parallel = strategy
        .and_then(|strategy| strategy.get("max-parallel"))
        .map_or_else(
            || {
                job_total.map_or(StaticValue::Unknown, |job_total| {
                    StaticValue::Number(job_total.to_string())
                })
            },
            |value| max_parallel_static_value(value, inputs),
        );
    (fail_fast, max_parallel)
}

fn max_parallel_static_value(value: &Value, inputs: &InputState) -> StaticValue {
    if let Some(value) = value.as_u64().filter(|value| *value > 0) {
        return StaticValue::Number(value.to_string());
    }
    let Some(expression) = value.as_str() else {
        return StaticValue::Unknown;
    };
    resolve_static_interpolations(expression, inputs, &EnvironmentState::default())
        .and_then(|resolved| serde_yaml::from_str::<Value>(&resolved).ok())
        .and_then(|value| value.as_u64())
        .filter(|value| *value > 0)
        .map(|value| StaticValue::Number(value.to_string()))
        .unwrap_or(StaticValue::Unknown)
}

fn fail_fast_valid_for_inputs(value: &Value, inputs: &InputState) -> bool {
    if value.is_bool() {
        return true;
    }
    value.as_str().is_some_and(|expression| {
        complete_expression_static_value(expression, inputs)
            .is_none_or(|value| matches!(value, StaticValue::Bool(_) | StaticValue::Unknown))
    })
}

fn max_parallel_valid_for_inputs(max_parallel: &Value, inputs: &InputState) -> bool {
    if max_parallel.as_u64().is_some_and(|value| value > 0) {
        return true;
    }
    let Some(expression) = max_parallel.as_str() else {
        return false;
    };
    resolve_static_interpolations(expression, inputs, &EnvironmentState::default()).is_none_or(
        |resolved| {
            serde_yaml::from_str::<Value>(&resolved)
                .ok()
                .and_then(|value| value.as_u64())
                .is_some_and(|value| value > 0)
        },
    )
}

fn strategy_fail_fast_expression_valid(value: &str) -> bool {
    complete_expression_contexts_available(value, STRATEGY_CONTEXTS)
        && !invalid_literal_from_json(value)
        && matches!(
            complete_expression_type(value),
            Some(StaticExpressionType::Boolean | StaticExpressionType::Dynamic)
        )
        && complete_literal_expression_value(value).is_none_or(|literal| literal.is_bool())
}

fn strategy_max_parallel_expression_valid(value: &str) -> bool {
    complete_expression_contexts_available(value, STRATEGY_CONTEXTS)
        && !invalid_literal_from_json(value)
        && matches!(
            complete_expression_type(value),
            Some(StaticExpressionType::Number | StaticExpressionType::Dynamic)
        )
        && complete_literal_expression_value(value)
            .is_none_or(|literal| literal.as_u64().is_some_and(|parallelism| parallelism > 0))
}
