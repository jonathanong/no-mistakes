use super::contracts::{input_contract_valid, normalized_name, workflow_call_contract_valid};
use super::{InputState, StaticValue};
use crate::codebase::workflow_topology::model::{
    JsonScalar, WorkflowCallContract, WorkflowCallInputType,
};
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};

mod bindings;
mod values;
use bindings::{binding_bool, binding_matches_type, normalized_bindings};
use values::{default_value, nonboolean_binding_value};

pub(crate) fn direct_inputs(contract: Option<&WorkflowCallContract>) -> Option<InputState> {
    // A workflow invoked directly by a repository event receives the declared
    // false/default values that GitHub assigns when workflow_call is not used.
    let Some(contract) = contract else {
        return Some(InputState::new());
    };
    if !workflow_call_contract_valid(contract) {
        return None;
    }
    Some(
        contract
            .inputs
            .keys()
            .map(|name| (normalized_name(name), StaticValue::Bool(false)))
            .collect(),
    )
}

pub(crate) fn callee_inputs(
    contract: Option<&WorkflowCallContract>,
    call_job: &Value,
    parent: &InputState,
) -> Option<InputState> {
    inputs_from_contract(contract?, call_job.get("with"), parent)
}

fn inputs_from_contract(
    contract: &WorkflowCallContract,
    bindings: Option<&Value>,
    parent: &InputState,
) -> Option<InputState> {
    if !input_contract_valid(contract) {
        return None;
    }
    let binding_map = match bindings {
        Some(Value::Mapping(mapping)) => Some(normalized_bindings(mapping)?),
        Some(_) => return None,
        None => None,
    };
    let declared_names: BTreeSet<String> = contract
        .inputs
        .keys()
        .map(|name| normalized_name(name))
        .collect();
    if binding_map
        .as_ref()
        .is_some_and(|mapping| mapping.keys().any(|name| !declared_names.contains(name)))
    {
        return None;
    }
    for (name, declaration) in &contract.inputs {
        let input_type = declaration
            .input_type
            .expect("validated workflow_call input type");
        let binding = binding_map
            .as_ref()
            .and_then(|mapping| mapping.get(&normalized_name(name)));
        if binding.is_none() && declaration.required {
            return None;
        }
        if binding.is_some_and(|binding| !binding_matches_type(binding, input_type, parent)) {
            return None;
        }
    }
    Some(
        contract
            .inputs
            .iter()
            .map(|(name, declaration)| {
                let input_type = declaration
                    .input_type
                    .expect("validated workflow_call input type");
                let binding = binding_map
                    .as_ref()
                    .and_then(|mapping| mapping.get(&normalized_name(name)));
                let state = match input_type {
                    WorkflowCallInputType::Boolean => binding
                        .map(|value| binding_bool(value, parent))
                        .unwrap_or_else(|| {
                            if let Some(JsonScalar::Bool(value)) = declaration.default.as_ref() {
                                StaticValue::Bool(*value)
                            } else {
                                StaticValue::Bool(false)
                            }
                        }),
                    WorkflowCallInputType::Number | WorkflowCallInputType::String => binding
                        .map(|value| nonboolean_binding_value(value, parent, input_type))
                        .unwrap_or_else(|| default_value(declaration.default.as_ref(), input_type)),
                };
                (normalized_name(name), state)
            })
            .collect(),
    )
}

pub(crate) fn callee_secrets_valid(contract: &WorkflowCallContract, call_job: &Value) -> bool {
    if !workflow_call_contract_valid(contract) {
        return false;
    }
    if call_job.get("secrets").and_then(Value::as_str) == Some("inherit") {
        return true;
    }
    let bindings = match call_job.get("secrets") {
        Some(Value::Mapping(mapping)) => match normalized_bindings(mapping) {
            Some(bindings) => bindings,
            None => return false,
        },
        Some(_) => return false,
        None => BTreeMap::new(),
    };
    let declared_names: BTreeSet<String> = contract
        .secrets
        .keys()
        .map(|name| normalized_name(name))
        .collect();
    if bindings.keys().any(|name| !declared_names.contains(name)) {
        return false;
    }
    if bindings.values().any(|value| {
        !matches!(
            *value,
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)
        )
    }) {
        return false;
    }
    contract.secrets.iter().all(|(name, declaration)| {
        !declaration.required || bindings.contains_key(&normalized_name(name))
    })
}

#[cfg(test)]
mod tests;
