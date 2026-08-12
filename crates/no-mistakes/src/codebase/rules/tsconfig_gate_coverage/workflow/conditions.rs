use serde_yaml::Value;
use std::collections::BTreeMap;

mod condition_values;
mod contracts;
mod environment;
mod evaluation;
mod functions;
mod input_value;
mod inputs;
mod literals;
mod logical;
mod resolution;
mod static_json;
#[cfg(test)]
mod static_json_tests;
mod static_values;
mod step_evaluation;
mod step_outcomes;

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) use contracts::valid_identifier;
pub(super) use environment::EnvironmentState;
pub(super) use evaluation::{
    continues_after_failed_need, continues_after_indeterminate_need, continues_after_skipped_need,
};
pub(super) use evaluation::{
    expression_bool, expression_bool_with_status_and_environment, job_statically_disabled,
    job_statically_enabled, job_statically_enforcing, job_statically_not_enforcing,
    job_timeout_minutes_validity, job_tolerates_failure, step_timeout_minutes_validity,
};
pub(super) use inputs::{
    callee_inputs, callee_secrets, direct_inputs, inputs_with_matrix_values,
    inputs_with_needs_results, inputs_with_static_strategy_position_values,
    inputs_with_strategy_configuration_values, MatrixState, SecretAvailability, SecretState,
};
use inputs::{
    event_action_value, event_base_ref_value, event_head_ref_value, event_name_value,
    event_ref_name_value, event_ref_type_value, pull_request_merged_value,
};
use resolution::condition_input_value;
pub(crate) use resolution::context_output_name;
pub(crate) use static_values::complete_expression_static_string_value;
pub(super) use step_evaluation::{
    continue_on_error_value, step_condition_with_status, step_continue_on_error_value_valid,
};
pub(super) use step_outcomes::StepOutcomes;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum StaticBool {
    False,
    True,
    TruthyNonBoolean,
    /// A statically-known expression error. It propagates through a condition
    /// so a valid sibling (for example `|| true`) cannot earn coverage credit.
    Invalid,
    Unknown,
}

#[derive(Clone, Copy)]
pub(super) struct ConditionStatus {
    success: StaticBool,
    failure: StaticBool,
}

impl ConditionStatus {
    const SUCCESS: Self = Self {
        success: StaticBool::True,
        failure: StaticBool::False,
    };

    const SKIPPED: Self = Self {
        success: StaticBool::False,
        failure: StaticBool::False,
    };

    const FAILURE: Self = Self {
        success: StaticBool::False,
        failure: StaticBool::True,
    };

    fn from_success(success: StaticBool) -> Self {
        Self {
            success,
            failure: success.negate(),
        }
    }
}

impl From<StaticBool> for ConditionStatus {
    fn from(success: StaticBool) -> Self {
        Self::from_success(success)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum StaticValue {
    Bool(bool),
    String(String),
    Number(String),
    Null,
    Sequence(Vec<Self>),
    /// A known static YAML mapping from a matrix axis. It is truthy in
    /// conditions but cannot be coerced to a GitHub string.
    Mapping,
    /// A mapping materialized from one matrix property. Its identity is stable
    /// within the activation, so repeated reads of that property compare equal.
    MatrixMapping(String),
    NonStringable,
    /// A context-free `fromJSON` call whose input is known malformed JSON.
    /// This is distinct from a dynamic value so condition evaluation can fail
    /// closed rather than allowing a sibling logical operand to earn credit.
    Invalid,
    Unknown,
}

pub(super) type InputState = BTreeMap<String, StaticValue>;

/// Resolves a complete expression only when its result is already known for
/// this activation. Callers that require a scalar can distinguish a known
/// object or array from a genuinely dynamic expression.
pub(crate) fn complete_expression_static_value(
    value: &str,
    inputs: &InputState,
) -> Option<StaticValue> {
    complete_expression_static_value_with_environment(value, inputs, &EnvironmentState::default())
}

pub(crate) fn complete_expression_static_value_with_environment(
    value: &str,
    inputs: &InputState,
    environment: &EnvironmentState,
) -> Option<StaticValue> {
    static_values::complete_expression_static_value_with_environment(value, inputs, environment)
}

pub(crate) fn resolve_static_interpolations(
    value: &str,
    inputs: &InputState,
    environment: &EnvironmentState,
) -> Option<String> {
    super::expressions::resolve_interpolations(value, |expression| {
        condition_input_value(expression, inputs, environment)
            .and_then(|value| value.function_string())
            .or_else(|| {
                resolution::secret_name(&format!("${{{{ {expression} }}}}")).and_then(|name| {
                    environment
                        .secret_value(name)
                        .and_then(|value| value.function_string())
                        .or_else(|| {
                            (environment.secret_availability(name) == SecretAvailability::Absent)
                                .then(String::new)
                        })
                })
            })
            .or_else(|| {
                super::expressions::complete_literal_expression_value(&format!(
                    "${{{{ {expression} }}}}"
                ))
                .and_then(static_value_string)
            })
    })
}

fn static_value_string(value: Value) -> Option<String> {
    match value {
        Value::Bool(value) => Some(value.to_string()),
        Value::Number(value) => Some(value.to_string()),
        Value::String(value) => Some(value),
        Value::Null => Some(String::new()),
        Value::Sequence(_) | Value::Mapping(_) | Value::Tagged(_) => None,
    }
}

fn static_yaml_value(value: Value) -> StaticValue {
    match value {
        Value::Bool(value) => StaticValue::Bool(value),
        Value::Number(value) => StaticValue::Number(value.to_string()),
        Value::String(value) => StaticValue::String(value),
        Value::Null => StaticValue::Null,
        Value::Sequence(_) | Value::Mapping(_) | Value::Tagged(_) => StaticValue::NonStringable,
    }
}

#[cfg(test)]
mod condition_values_edge_tests;
#[cfg(test)]
mod condition_values_tests;
#[cfg(test)]
mod contains_tests;
#[cfg(test)]
mod enforcement_tests;
#[cfg(test)]
mod format_tests;
#[cfg(test)]
mod join_tests;
#[cfg(test)]
mod literal_from_json_tests;
#[cfg(test)]
mod matrix_tests;
#[cfg(test)]
mod relational_tests;
#[cfg(test)]
mod review_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod timeout_tests;
#[cfg(test)]
mod to_json_tests;
