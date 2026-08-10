use super::{expression_bool, InputState, StaticBool};
use crate::codebase::workflow_topology::model::{
    JsonScalar, WorkflowCallContract, WorkflowCallInputType,
};
use serde_yaml::Value;

pub(crate) fn direct_inputs(contract: Option<&WorkflowCallContract>) -> InputState {
    contract
        .into_iter()
        .flat_map(|contract| &contract.inputs)
        .filter(|(_, declaration)| declaration.input_type == Some(WorkflowCallInputType::Boolean))
        .map(|(name, _)| (name.clone(), StaticBool::False))
        .collect()
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
    let binding_map = match bindings {
        Some(Value::Mapping(mapping)) => Some(mapping),
        Some(_) => return None,
        None => None,
    };
    if binding_map.is_some_and(|mapping| {
        mapping
            .keys()
            .filter_map(Value::as_str)
            .any(|name| !contract.inputs.contains_key(name))
            || mapping.keys().any(|key| key.as_str().is_none())
    }) {
        return None;
    }
    for (name, declaration) in &contract.inputs {
        let input_type = declaration.input_type?;
        let binding = binding_map.and_then(|mapping| mapping.get(Value::String(name.clone())));
        if binding.is_none() && declaration.required && declaration.default.is_none() {
            return None;
        }
        if declaration
            .default
            .as_ref()
            .is_some_and(|default| !default_matches_type(default, input_type))
        {
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
            .filter(|(_, declaration)| {
                declaration.input_type == Some(WorkflowCallInputType::Boolean)
            })
            .map(|(name, declaration)| {
                let binding =
                    binding_map.and_then(|mapping| mapping.get(Value::String(name.clone())));
                let state = binding
                    .map(|value| binding_bool(value, parent))
                    .unwrap_or_else(|| {
                        if let Some(JsonScalar::Bool(value)) = declaration.default.as_ref() {
                            StaticBool::from(*value)
                        } else {
                            StaticBool::False
                        }
                    });
                (name.clone(), state)
            })
            .collect(),
    )
}

fn default_matches_type(value: &JsonScalar, input_type: WorkflowCallInputType) -> bool {
    matches!(
        (input_type, value),
        (WorkflowCallInputType::Boolean, JsonScalar::Bool(_))
            | (WorkflowCallInputType::Number, JsonScalar::Number(_))
            | (WorkflowCallInputType::String, JsonScalar::Text(_))
    )
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
