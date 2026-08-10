use super::contracts::{input_contract_valid, normalized_name, unique_contract_names};
use super::{expression_bool, InputState, StaticBool};
use crate::codebase::workflow_topology::model::{
    JsonScalar, WorkflowCallContract, WorkflowCallInputType,
};
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn direct_inputs(contract: Option<&WorkflowCallContract>) -> Option<InputState> {
    // A workflow invoked directly by a repository event receives the declared
    // false/default values that GitHub assigns when workflow_call is not used.
    let Some(contract) = contract else {
        return Some(InputState::new());
    };
    if !input_contract_valid(contract) || !unique_contract_names(contract.secrets.keys()) {
        return None;
    }
    Some(
        contract
            .inputs
            .iter()
            .map(|(name, declaration)| {
                let state = match declaration
                    .input_type
                    .expect("validated workflow_call input type")
                {
                    WorkflowCallInputType::Boolean => StaticBool::False,
                    WorkflowCallInputType::Number | WorkflowCallInputType::String => {
                        StaticBool::Unknown
                    }
                };
                (normalized_name(name), state)
            })
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
        if binding.is_some_and(|binding| !binding_matches_type(binding, input_type)) {
            return None;
        }
    }
    Some(
        contract
            .inputs
            .iter()
            .map(|(name, declaration)| {
                let binding = binding_map
                    .as_ref()
                    .and_then(|mapping| mapping.get(&normalized_name(name)));
                let state = match declaration
                    .input_type
                    .expect("validated workflow_call input type")
                {
                    WorkflowCallInputType::Boolean => binding
                        .map(|value| binding_bool(value, parent))
                        .unwrap_or_else(|| {
                            if let Some(JsonScalar::Bool(value)) = declaration.default.as_ref() {
                                StaticBool::from(*value)
                            } else {
                                StaticBool::False
                            }
                        }),
                    WorkflowCallInputType::Number | WorkflowCallInputType::String => {
                        StaticBool::Unknown
                    }
                };
                (normalized_name(name), state)
            })
            .collect(),
    )
}

pub(crate) fn callee_secrets_valid(contract: &WorkflowCallContract, call_job: &Value) -> bool {
    if !unique_contract_names(contract.secrets.keys()) {
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

fn normalized_bindings(mapping: &serde_yaml::Mapping) -> Option<BTreeMap<String, &Value>> {
    let mut bindings = BTreeMap::new();
    for (name, value) in mapping {
        let name = normalized_name(name.as_str()?);
        if bindings.insert(name, value).is_some() {
            return None;
        }
    }
    Some(bindings)
}

fn binding_matches_type(value: &Value, input_type: WorkflowCallInputType) -> bool {
    if value
        .as_str()
        .is_some_and(|text| is_complete_expression(text.trim()))
    {
        return true;
    }
    matches!(
        (input_type, value),
        (WorkflowCallInputType::Boolean, Value::Bool(_))
            | (WorkflowCallInputType::Number, Value::Number(_))
            | (WorkflowCallInputType::String, Value::String(_))
    )
}

fn is_complete_expression(value: &str) -> bool {
    value.starts_with("${{") && value.ends_with("}}")
}

fn binding_bool(value: &Value, parent: &InputState) -> StaticBool {
    if let Some(value) = value.as_bool() {
        StaticBool::from(value)
    } else {
        expression_bool(value.as_str().unwrap_or_default(), parent)
    }
}
