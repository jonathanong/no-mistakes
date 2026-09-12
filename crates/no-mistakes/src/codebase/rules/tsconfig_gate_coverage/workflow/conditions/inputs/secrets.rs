use super::super::contracts::{normalized_name, workflow_call_contract_valid};
use super::bindings::normalized_bindings;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::{
    complete_expression_contexts_available, REUSABLE_CALL_SECRET_BINDING_CONTEXTS,
};
use crate::codebase::workflow_topology::model::WorkflowCallContract;
use serde_yaml::Value;
use std::collections::BTreeMap;

use super::super::{InputState, StaticValue};

/// Secrets known to have crossed the current reusable-workflow boundary.
///
/// A directly triggered workflow has all repository secrets available. Explicit
/// bindings supply only their destination names; `secrets: inherit` preserves
/// the caller's availability state.
#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct SecretState {
    values: BTreeMap<String, StaticValue>,
    all: bool,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum SecretAvailability {
    Absent,
    Available,
}

impl SecretState {
    pub(crate) fn direct() -> Self {
        Self {
            values: BTreeMap::new(),
            all: true,
        }
    }

    pub(crate) fn unavailable() -> Self {
        Self::reusable(BTreeMap::new(), false)
    }

    fn reusable(values: BTreeMap<String, StaticValue>, all: bool) -> Self {
        Self { values, all }
    }

    pub(crate) fn availability(&self, name: &str) -> SecretAvailability {
        if self.all || self.values.contains_key(&normalized_name(name)) {
            SecretAvailability::Available
        } else {
            SecretAvailability::Absent
        }
    }

    pub(crate) fn value(&self, name: &str) -> Option<StaticValue> {
        self.values
            .get(&normalized_name(name))
            .cloned()
            .or_else(|| self.all.then_some(StaticValue::Unknown))
    }
}

pub(crate) fn callee_secrets(
    contract: &WorkflowCallContract,
    call_job: &Value,
    parent: &SecretState,
    inputs: &InputState,
) -> Option<SecretState> {
    if !workflow_call_contract_valid(contract) {
        return None;
    }
    let (values, all) = if call_job.get("secrets").and_then(Value::as_str) == Some("inherit") {
        (parent.values.clone(), parent.all)
    } else {
        (
            explicit_secret_bindings(contract, call_job, parent, inputs)?,
            false,
        )
    };
    contract
        .secrets
        .iter()
        .filter(|(_, declaration)| declaration.required)
        .all(|(name, _)| all || values.contains_key(&normalized_name(name)))
        .then(|| SecretState::reusable(values, all))
}

fn explicit_secret_bindings(
    contract: &WorkflowCallContract,
    call_job: &Value,
    parent: &SecretState,
    inputs: &InputState,
) -> Option<BTreeMap<String, StaticValue>> {
    let bindings = match call_job.get("secrets") {
        Some(Value::Mapping(mapping)) => normalized_bindings(mapping)?,
        Some(_) => return None,
        None => BTreeMap::new(),
    };
    let declared_names = contract
        .secrets
        .keys()
        .map(|name| normalized_name(name))
        .collect::<std::collections::BTreeSet<_>>();
    if bindings.keys().any(|name| !declared_names.contains(name))
        || bindings.values().any(|value| {
            !matches!(
                *value,
                Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)
            )
        })
        || bindings.values().any(|value| {
            value.as_str().is_some_and(|value| {
                value.contains("${{")
                    && !complete_expression_contexts_available(
                        value,
                        REUSABLE_CALL_SECRET_BINDING_CONTEXTS,
                    )
            })
        })
    {
        return None;
    }
    Some(
        bindings
            .into_iter()
            .filter_map(|(destination, value)| {
                let source = value
                    .as_str()
                    .and_then(super::super::resolution::secret_name);
                let value = if let Some(source) = source {
                    parent.value(source)?
                } else {
                    super::values::forwarded_input_value(value, inputs)
                        .or_else(|| scalar_secret_value(value))
                        .unwrap_or(StaticValue::Unknown)
                };
                Some((destination, value))
            })
            .collect(),
    )
}

fn scalar_secret_value(value: &Value) -> Option<StaticValue> {
    match value {
        Value::Null => Some(StaticValue::String(String::new())),
        Value::Bool(value) => Some(StaticValue::String(value.to_string())),
        Value::Number(value) => Some(StaticValue::String(value.to_string())),
        Value::String(value) if !value.contains("${{") => Some(StaticValue::String(value.clone())),
        Value::String(_) | Value::Sequence(_) | Value::Mapping(_) | Value::Tagged(_) => None,
    }
}

#[cfg(test)]
mod tests;
