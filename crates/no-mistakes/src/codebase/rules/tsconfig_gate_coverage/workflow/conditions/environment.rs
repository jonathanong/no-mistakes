use super::{
    resolution::{condition_input_value, input_name, secret_name},
    SecretAvailability, SecretState, StaticValue, StepOutcomes,
};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::complete_literal_expression_value;
use serde_yaml::Value;
use std::collections::BTreeMap;

#[derive(Clone)]
pub(crate) struct EnvironmentState {
    values: BTreeMap<String, StaticValue>,
    secrets: SecretState,
    step_outcomes: StepOutcomes,
    runner_os: StaticValue,
}

impl Default for EnvironmentState {
    fn default() -> Self {
        Self {
            values: BTreeMap::new(),
            secrets: SecretState::direct(),
            step_outcomes: StepOutcomes::default(),
            runner_os: StaticValue::Unknown,
        }
    }
}

impl EnvironmentState {
    pub(crate) fn from_workflow(
        workflow: &Value,
        secrets: &SecretState,
        inputs: &super::InputState,
    ) -> Self {
        Self {
            values: values(
                workflow.get("env"),
                secrets,
                inputs,
                &EnvironmentState::default(),
            ),
            secrets: secrets.clone(),
            step_outcomes: StepOutcomes::default(),
            runner_os: StaticValue::Unknown,
        }
    }

    pub(crate) fn with_job(&self, job: &Value, inputs: &super::InputState) -> Self {
        self.with_scope(job, inputs)
    }

    pub(crate) fn with_step(&self, step: &Value, inputs: &super::InputState) -> Self {
        self.with_scope(step, inputs)
    }

    pub(crate) fn value(&self, name: &str) -> Option<StaticValue> {
        self.values.get(&name.to_lowercase()).cloned()
    }

    pub(crate) fn step_outcome(&self, id: &str) -> StaticValue {
        self.step_outcomes.value(id)
    }

    pub(crate) fn runner_os(&self) -> StaticValue {
        self.runner_os.clone()
    }

    pub(crate) fn has_invalid_value(&self) -> bool {
        self.values.values().any(|value| {
            matches!(
                value,
                StaticValue::Sequence(_)
                    | StaticValue::Mapping
                    | StaticValue::NonStringable
                    | StaticValue::Invalid
            )
        })
    }

    pub(crate) fn with_runner_os(&self, runner_os: Option<&str>) -> Self {
        Self {
            values: self.values.clone(),
            secrets: self.secrets.clone(),
            step_outcomes: self.step_outcomes.clone(),
            runner_os: runner_os
                .map(|runner_os| StaticValue::String(runner_os.to_string()))
                .unwrap_or(StaticValue::Unknown),
        }
    }

    pub(crate) fn with_step_outcomes(&self, step_outcomes: &StepOutcomes) -> Self {
        Self {
            values: self.values.clone(),
            secrets: self.secrets.clone(),
            step_outcomes: step_outcomes.clone(),
            runner_os: self.runner_os.clone(),
        }
    }

    pub(crate) fn secret_availability(&self, name: &str) -> SecretAvailability {
        self.secrets.availability(name)
    }

    fn with_scope(&self, scope: &Value, inputs: &super::InputState) -> Self {
        let mut environment_values = self.values.clone();
        environment_values.extend(values(scope.get("env"), &self.secrets, inputs, self));
        Self {
            values: environment_values,
            secrets: self.secrets.clone(),
            step_outcomes: self.step_outcomes.clone(),
            runner_os: self.runner_os.clone(),
        }
    }
}

fn values(
    value: Option<&Value>,
    secrets: &SecretState,
    inputs: &super::InputState,
    environment: &EnvironmentState,
) -> BTreeMap<String, StaticValue> {
    value
        .and_then(Value::as_mapping)
        .into_iter()
        .flatten()
        .filter_map(|(name, raw_value)| {
            Some((
                name.as_str()?.to_lowercase(),
                environment_value(raw_value, secrets, inputs, environment),
            ))
        })
        .collect()
}

fn environment_value(
    value: &Value,
    secrets: &SecretState,
    inputs: &super::InputState,
    environment: &EnvironmentState,
) -> StaticValue {
    match value {
        Value::Bool(value) => StaticValue::String(value.to_string()),
        Value::Number(value) => StaticValue::String(value.to_string()),
        Value::String(value) if !value.contains("${{") => StaticValue::String(value.clone()),
        Value::String(value) => secret_name(value)
            .and_then(|name| {
                (secrets.availability(name) == SecretAvailability::Absent)
                    .then(|| StaticValue::String(String::new()))
            })
            .or_else(|| complete_literal_expression_value(value).map(string_value))
            .or_else(|| expression_input_value(value, inputs, environment).map(string_static_value))
            .unwrap_or(StaticValue::Unknown),
        Value::Null | Value::Sequence(_) | Value::Mapping(_) | Value::Tagged(_) => {
            StaticValue::Unknown
        }
    }
}

fn expression_input_value(
    value: &str,
    inputs: &super::InputState,
    environment: &EnvironmentState,
) -> Option<StaticValue> {
    let expression = value.trim().strip_prefix("${{")?.strip_suffix("}}")?.trim();
    if let Some(name) = input_name(expression) {
        return Some(
            inputs
                .get(&name.to_lowercase())
                .cloned()
                // GitHub resolves a missing input in an environment value to
                // the empty string, rather than preserving condition false.
                .unwrap_or_else(|| StaticValue::String(String::new())),
        );
    }
    condition_input_value(expression, inputs, environment)
}

fn string_value(value: Value) -> StaticValue {
    match value {
        Value::Bool(value) => StaticValue::String(value.to_string()),
        Value::Number(value) => StaticValue::String(value.to_string()),
        Value::String(value) => StaticValue::String(value),
        Value::Null => StaticValue::String(String::new()),
        Value::Sequence(_) | Value::Mapping(_) | Value::Tagged(_) => StaticValue::Invalid,
    }
}

fn string_static_value(value: StaticValue) -> StaticValue {
    match value {
        StaticValue::Bool(value) => StaticValue::String(value.to_string()),
        StaticValue::Number(value) | StaticValue::String(value) => StaticValue::String(value),
        StaticValue::Null => StaticValue::String(String::new()),
        StaticValue::Sequence(_)
        | StaticValue::Mapping
        | StaticValue::NonStringable
        | StaticValue::Invalid => value,
        StaticValue::Unknown => StaticValue::Unknown,
    }
}

#[cfg(test)]
mod tests;
