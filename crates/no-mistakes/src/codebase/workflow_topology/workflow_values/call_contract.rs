//! `workflow_call:` contract parsing (`inputs`/`secrets`/`outputs`), split
//! out of [`super`] to stay under the crate's per-file line limit.
//! Re-exported by [`super`] so `workflow_values::parse_workflow_call` keeps
//! working unchanged for every external caller.

use super::super::model;
use super::super::value_primitives;
use super::key_name;
use serde_yaml::Value;
use std::collections::BTreeMap;

pub fn parse_workflow_call(value: Option<&Value>) -> Option<model::WorkflowCallContract> {
    let is_callable = match value {
        Some(Value::String(text)) => text == "workflow_call",
        Some(Value::Sequence(items)) => items
            .iter()
            .any(|item| item.as_str() == Some("workflow_call")),
        Some(v) if value_primitives::is_record(Some(v)) => v.get("workflow_call").is_some(),
        _ => false,
    };
    if !is_callable {
        return None;
    }
    // The string/array forms set `config = null` in the TS engine; only the
    // mapping form (`on: { workflow_call: {...} }`) carries a real config.
    let config = match value {
        Some(v) if value_primitives::is_record(Some(v)) => v.get("workflow_call"),
        _ => None,
    };
    let mapping = config.filter(|c| value_primitives::is_record(Some(c)));
    Some(model::WorkflowCallContract {
        inputs: parse_declarations(
            mapping.and_then(|m| m.get("inputs")),
            parse_workflow_call_input,
        ),
        secrets: parse_declarations(
            mapping.and_then(|m| m.get("secrets")),
            parse_workflow_call_secret,
        ),
        outputs: parse_declarations(
            mapping.and_then(|m| m.get("outputs")),
            parse_workflow_call_output,
        ),
    })
}

fn parse_declarations<T>(
    value: Option<&Value>,
    parse: impl Fn(Option<&Value>) -> T,
) -> BTreeMap<String, T> {
    let Some(Value::Mapping(mapping)) = value else {
        return BTreeMap::new();
    };
    mapping
        .iter()
        .filter_map(|(key, declaration)| Some((key_name(key)?, parse(Some(declaration)))))
        .collect()
}

fn parse_workflow_call_input(value: Option<&Value>) -> model::WorkflowCallInput {
    let input_type = value
        .and_then(|v| v.get("type"))
        .and_then(Value::as_str)
        .and_then(|text| match text {
            "boolean" => Some(model::WorkflowCallInputType::Boolean),
            "number" => Some(model::WorkflowCallInputType::Number),
            "string" => Some(model::WorkflowCallInputType::String),
            _ => None,
        });
    let required = value
        .and_then(|v| v.get("required"))
        .and_then(Value::as_bool)
        == Some(true);
    let default = value.and_then(|v| v.get("default")).and_then(scalar_value);
    let description = value
        .and_then(|v| v.get("description"))
        .and_then(Value::as_str)
        .map(str::to_string);
    model::WorkflowCallInput {
        input_type,
        required,
        default,
        description,
    }
}

fn parse_workflow_call_secret(value: Option<&Value>) -> model::WorkflowCallSecret {
    model::WorkflowCallSecret {
        required: value
            .and_then(|v| v.get("required"))
            .and_then(Value::as_bool)
            == Some(true),
        description: value
            .and_then(|v| v.get("description"))
            .and_then(Value::as_str)
            .map(str::to_string),
    }
}

fn parse_workflow_call_output(value: Option<&Value>) -> model::WorkflowCallOutput {
    model::WorkflowCallOutput {
        value: value
            .and_then(|v| v.get("value"))
            .and_then(Value::as_str)
            .map(str::to_string),
        description: value
            .and_then(|v| v.get("description"))
            .and_then(Value::as_str)
            .map(str::to_string),
    }
}

pub(super) fn scalar_record(value: Option<&Value>) -> BTreeMap<String, model::JsonScalar> {
    let Some(Value::Mapping(mapping)) = value else {
        return BTreeMap::new();
    };
    mapping
        .iter()
        .filter_map(|(key, item)| Some((key_name(key)?, scalar_value(item)?)))
        .collect()
}

fn scalar_value(value: &Value) -> Option<model::JsonScalar> {
    match value {
        Value::String(text) => Some(model::JsonScalar::Text(text.clone())),
        Value::Bool(flag) => Some(model::JsonScalar::Bool(*flag)),
        Value::Number(number) => {
            value_primitives::yaml_number_to_json(number).map(model::JsonScalar::Number)
        }
        _ => None,
    }
}
