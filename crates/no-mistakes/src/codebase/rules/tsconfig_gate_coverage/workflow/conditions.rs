use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};

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

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) use contracts::valid_identifier;
pub(super) use environment::EnvironmentState;
use evaluation::{continues_after_skipped_need, static_bool};
pub(super) use evaluation::{
    expression_bool, expression_bool_with_status_and_environment, job_timeout_minutes_enforced,
    statically_not_enforcing, statically_not_enforcing_with_environment,
    step_timeout_minutes_enforced,
};
pub(super) use inputs::{
    callee_inputs, callee_secrets, direct_inputs, inputs_with_matrix_values,
    inputs_with_needs_results, MatrixState, SecretAvailability, SecretState,
};
use inputs::{event_action_value, event_name_value};
use resolution::condition_input_value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum StaticBool {
    False,
    True,
    TruthyNonBoolean,
    Unknown,
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

pub(super) fn statically_skipped_jobs(
    jobs: &serde_yaml::Mapping,
    initial_skipped: &BTreeSet<String>,
    matrix_inputs: impl Fn(&Value, &Value) -> Vec<InputState>,
) -> BTreeSet<String> {
    let mut skipped = initial_skipped.clone();
    loop {
        let mut changed = false;
        for (raw_job_id, job) in jobs {
            let job_id = super::normalized_job_id(raw_job_id).expect("validated scalar job ID");
            let inputs = matrix_inputs(raw_job_id, job)
                .into_iter()
                .map(|inputs| inputs_with_needs_results(&inputs, job, &skipped))
                .collect::<Vec<_>>();
            let directly_disabled = !inputs.is_empty()
                && inputs
                    .iter()
                    .all(|inputs| static_bool(job.get("if"), inputs) == StaticBool::False);
            let blocked_by_need = !inputs
                .iter()
                .any(|inputs| continues_after_skipped_need(job, inputs))
                && crate::codebase::workflow_topology::value_primitives::string_list(
                    job.get("needs"),
                )
                .iter()
                .any(|need| skipped.contains(&need.to_lowercase()));
            if (directly_disabled || blocked_by_need) && skipped.insert(job_id) {
                changed = true;
            }
        }
        if !changed {
            return skipped;
        }
    }
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
