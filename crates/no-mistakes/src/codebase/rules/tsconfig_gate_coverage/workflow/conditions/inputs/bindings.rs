use super::super::contracts::normalized_name;
use super::super::{expression_bool, InputState, StaticBool, StaticValue};
use super::values::forwarded_input_value;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::{
    interpolated_expression_contexts_available, typed_scalar_expression_contexts_available,
    StaticExpressionType,
};
use crate::codebase::workflow_topology::model::WorkflowCallInputType;
use serde_yaml::Value;
use std::collections::BTreeMap;

const REUSABLE_CALL_INPUT_CONTEXTS: &[&str] =
    &["github", "needs", "strategy", "matrix", "inputs", "vars"];

pub(super) fn normalized_bindings(
    mapping: &serde_yaml::Mapping,
) -> Option<BTreeMap<String, &Value>> {
    let mut bindings = BTreeMap::new();
    for (name, value) in mapping {
        let name = normalized_name(name.as_str()?);
        if bindings.insert(name, value).is_some() {
            return None;
        }
    }
    Some(bindings)
}

pub(super) fn binding_matches_type(
    value: &Value,
    input_type: WorkflowCallInputType,
    parent: &InputState,
) -> bool {
    if let Some(value) = value.as_str() {
        let expected = match input_type {
            WorkflowCallInputType::Boolean => StaticExpressionType::Boolean,
            WorkflowCallInputType::Number => StaticExpressionType::Number,
            WorkflowCallInputType::String => StaticExpressionType::String,
        };
        if typed_scalar_expression_contexts_available(value, REUSABLE_CALL_INPUT_CONTEXTS, expected)
        {
            return forwarded_input_value(&Value::String(value.to_string()), parent).is_none_or(
                |value| {
                    matches!(
                        (input_type, value),
                        (_, StaticValue::Unknown)
                            | (WorkflowCallInputType::Boolean, StaticValue::Bool(_))
                            | (WorkflowCallInputType::Number, StaticValue::Number(_))
                            | (WorkflowCallInputType::String, StaticValue::String(_))
                    )
                },
            );
        }
        if value.trim().starts_with("${{") && value.trim().ends_with("}}") {
            return false;
        }
    }
    if let Some(value) = value.as_str().filter(|value| value.contains("${{")) {
        return input_type == WorkflowCallInputType::String
            && interpolated_expression_contexts_available(value, REUSABLE_CALL_INPUT_CONTEXTS);
    }
    matches!(
        (input_type, value),
        (WorkflowCallInputType::Boolean, Value::Bool(_))
            | (WorkflowCallInputType::Number, Value::Number(_))
            | (WorkflowCallInputType::String, Value::String(_))
    )
}

#[cfg(test)]
mod tests;

pub(super) fn binding_bool(value: &Value, parent: &InputState) -> StaticValue {
    if let Some(value) = value.as_bool() {
        StaticValue::Bool(value)
    } else if let Some(value) = forwarded_input_value(value, parent) {
        match value {
            StaticValue::Bool(value) => StaticValue::Bool(value),
            StaticValue::Unknown => StaticValue::Unknown,
            StaticValue::String(_)
            | StaticValue::Number(_)
            | StaticValue::Null
            | StaticValue::Sequence(_) => StaticValue::Unknown,
        }
    } else {
        match expression_bool(value.as_str().unwrap_or_default(), parent) {
            StaticBool::False => StaticValue::Bool(false),
            StaticBool::True => StaticValue::Bool(true),
            StaticBool::TruthyNonBoolean | StaticBool::Unknown => StaticValue::Unknown,
        }
    }
}
