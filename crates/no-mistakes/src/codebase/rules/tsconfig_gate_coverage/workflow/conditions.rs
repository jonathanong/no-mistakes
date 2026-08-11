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
mod step_evaluation;

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) use contracts::valid_identifier;
pub(super) use environment::EnvironmentState;
#[cfg(test)]
use evaluation::static_bool;
pub(super) use evaluation::{continues_after_failed_need, continues_after_skipped_need};
pub(super) use evaluation::{
    expression_bool, expression_bool_with_status_and_environment, job_statically_disabled,
    job_statically_enabled, job_statically_enforcing, job_statically_not_enforcing,
    job_timeout_minutes_enforced, step_timeout_minutes_enforced,
};
pub(super) use inputs::{
    callee_inputs, callee_secrets, direct_inputs, inputs_with_matrix_values,
    inputs_with_needs_results, MatrixState, SecretAvailability, SecretState,
};
use inputs::{event_action_value, event_name_value};
use resolution::condition_input_value;
pub(super) use step_evaluation::{continue_on_error_enabled, step_condition_with_status};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum StaticBool {
    False,
    True,
    TruthyNonBoolean,
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
    NonStringable,
    Unknown,
}

pub(super) type InputState = BTreeMap<String, StaticValue>;

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
                    (environment.secret_availability(name) == SecretAvailability::Absent)
                        .then(String::new)
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

#[cfg(test)]
mod condition_values_tests;
#[cfg(test)]
mod contains_tests;
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
