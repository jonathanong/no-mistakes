use super::contracts::{input_contract_valid, normalized_name, workflow_call_contract_valid};
use super::{InputState, StaticValue};
use crate::codebase::workflow_topology::model::{
    JsonScalar, WorkflowCallContract, WorkflowCallInputType,
};
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};

mod bindings;
mod secrets;
mod values;
use bindings::{binding_bool, binding_matches_type, normalized_bindings};
pub(crate) use secrets::{callee_secrets, SecretState};
use values::{default_value, nonboolean_binding_value};

pub(super) const EVENT_NAME_KEY: &str = "\0github.event_name";

pub(super) fn event_name_value(inputs: &InputState) -> Option<StaticValue> {
    inputs.get(EVENT_NAME_KEY).cloned()
}

pub(super) const MATRIX_VALUE_PREFIX: &str = "\0matrix.";
const DYNAMIC_MATRIX_KEY: &str = "\0matrix.dynamic";

#[derive(Clone, Copy)]
pub(crate) enum MatrixState {
    Static,
    Dynamic,
}

pub(crate) fn inputs_with_matrix_values(
    parent: &InputState,
    matrix_values: &BTreeMap<String, Value>,
    matrix_state: MatrixState,
) -> InputState {
    let mut inputs = parent.clone();
    for (name, value) in matrix_values {
        if let Some(value) = values::matrix_axis_value(value) {
            inputs.insert(
                format!("{MATRIX_VALUE_PREFIX}{}", name.to_lowercase()),
                value,
            );
        }
    }
    if matches!(matrix_state, MatrixState::Dynamic) {
        inputs.insert(DYNAMIC_MATRIX_KEY.to_string(), StaticValue::Unknown);
    }
    inputs
}

pub(super) fn matrix_property_is_dynamic(inputs: &InputState) -> bool {
    inputs.contains_key(DYNAMIC_MATRIX_KEY)
}

pub(crate) fn direct_inputs(
    contract: Option<&WorkflowCallContract>,
    event_name: &str,
) -> Option<InputState> {
    // A workflow invoked directly by a repository event receives the declared
    // false/default values that GitHub assigns when workflow_call is not used.
    let Some(contract) = contract else {
        return Some(InputState::from([(
            EVENT_NAME_KEY.to_string(),
            StaticValue::String(event_name.to_string()),
        )]));
    };
    if !workflow_call_contract_valid(contract) {
        return None;
    }
    let mut inputs: InputState = contract
        .inputs
        .iter()
        .map(|(name, declaration)| {
            let input_type = declaration
                .input_type
                .expect("validated workflow_call input type");
            (normalized_name(name), default_value(None, input_type))
        })
        .collect();
    inputs.insert(
        EVENT_NAME_KEY.to_string(),
        StaticValue::String(event_name.to_string()),
    );
    Some(inputs)
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
    let mut inputs: InputState = contract
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
                    .unwrap_or_else(|| default_value(declaration.default.as_ref(), input_type)),
                WorkflowCallInputType::Number | WorkflowCallInputType::String => binding
                    .map(|value| nonboolean_binding_value(value, parent, input_type))
                    .unwrap_or_else(|| default_value(declaration.default.as_ref(), input_type)),
            };
            (normalized_name(name), state)
        })
        .collect();
    inputs.insert(
        EVENT_NAME_KEY.to_string(),
        parent.get(EVENT_NAME_KEY)?.clone(),
    );
    Some(inputs)
}

#[cfg(test)]
mod matrix_tests;
#[cfg(test)]
mod tests;
