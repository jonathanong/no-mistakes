use super::{resolution::secret_name, SecretAvailability, SecretState, StaticValue};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::complete_literal_expression_value;
use serde_yaml::Value;
use std::collections::BTreeMap;

pub(crate) struct EnvironmentState {
    values: BTreeMap<String, StaticValue>,
    secrets: SecretState,
}

impl Default for EnvironmentState {
    fn default() -> Self {
        Self {
            values: BTreeMap::new(),
            secrets: SecretState::direct(),
        }
    }
}

impl EnvironmentState {
    pub(crate) fn from_workflow(workflow: &Value, secrets: &SecretState) -> Self {
        Self {
            values: values(workflow.get("env"), secrets),
            secrets: secrets.clone(),
        }
    }

    pub(crate) fn with_job(&self, job: &Value) -> Self {
        self.with_scope(job)
    }

    pub(crate) fn with_step(&self, step: &Value) -> Self {
        self.with_scope(step)
    }

    pub(crate) fn value(&self, name: &str) -> Option<StaticValue> {
        self.values.get(&name.to_lowercase()).cloned()
    }

    fn with_scope(&self, scope: &Value) -> Self {
        let mut environment_values = self.values.clone();
        environment_values.extend(values(scope.get("env"), &self.secrets));
        Self {
            values: environment_values,
            secrets: self.secrets.clone(),
        }
    }
}

fn values(value: Option<&Value>, secrets: &SecretState) -> BTreeMap<String, StaticValue> {
    value
        .and_then(Value::as_mapping)
        .into_iter()
        .flatten()
        .filter_map(|(name, raw_value)| {
            Some((
                name.as_str()?.to_lowercase(),
                environment_value(raw_value, secrets),
            ))
        })
        .collect()
}

fn environment_value(value: &Value, secrets: &SecretState) -> StaticValue {
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
            .unwrap_or(StaticValue::Unknown),
        Value::Null | Value::Sequence(_) | Value::Mapping(_) | Value::Tagged(_) => {
            StaticValue::Unknown
        }
    }
}

fn string_value(value: Value) -> StaticValue {
    match value {
        Value::Bool(value) => StaticValue::String(value.to_string()),
        Value::Number(value) => StaticValue::String(value.to_string()),
        Value::String(value) => StaticValue::String(value),
        Value::Null => StaticValue::String(String::new()),
        Value::Sequence(_) | Value::Mapping(_) | Value::Tagged(_) => StaticValue::Unknown,
    }
}
