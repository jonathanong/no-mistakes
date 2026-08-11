use crate::codebase::rules::tsconfig_gate_coverage::workflow::{
    conditions::{complete_expression_static_value, InputState, StaticValue},
    expressions::{complete_literal_expression_value, condition_function_call, Function},
};
use serde_yaml::Value;

pub(super) enum ResolvedRootMatrix {
    Mapping(serde_yaml::Mapping),
    NonMapping,
    Dynamic,
}

pub(super) fn resolve(expression: &str, inputs: &InputState) -> ResolvedRootMatrix {
    if let Some(value) = complete_literal_expression_value(expression) {
        return yaml_value(value);
    }
    let Some(body) = expression
        .trim()
        .strip_prefix("${{")
        .and_then(|value| value.strip_suffix("}}"))
        .map(str::trim)
    else {
        return ResolvedRootMatrix::Dynamic;
    };
    let Some(call) = condition_function_call(body) else {
        return complete_expression_static_value(expression, inputs)
            .map_or(ResolvedRootMatrix::Dynamic, static_value);
    };
    if call.function != Function::FromJson {
        return ResolvedRootMatrix::Dynamic;
    }
    let Some(argument) = call.arguments.first() else {
        return ResolvedRootMatrix::Dynamic;
    };
    let argument = format!("${{{{ {argument} }}}}");
    match complete_expression_static_value(&argument, inputs) {
        Some(StaticValue::Unknown) | None => ResolvedRootMatrix::Dynamic,
        Some(value) => value
            .function_string()
            .map_or(ResolvedRootMatrix::NonMapping, |value| {
                serde_json::from_str::<serde_json::Value>(&value)
                    .ok()
                    .and_then(|value| serde_yaml::to_value(value).ok())
                    .map_or(ResolvedRootMatrix::NonMapping, yaml_value)
            }),
    }
}

fn static_value(value: StaticValue) -> ResolvedRootMatrix {
    match value {
        StaticValue::Unknown => ResolvedRootMatrix::Dynamic,
        StaticValue::Bool(_)
        | StaticValue::String(_)
        | StaticValue::Number(_)
        | StaticValue::Null
        | StaticValue::Sequence(_)
        | StaticValue::Mapping
        | StaticValue::NonStringable => ResolvedRootMatrix::NonMapping,
    }
}

fn yaml_value(value: Value) -> ResolvedRootMatrix {
    match value {
        Value::Mapping(matrix) => ResolvedRootMatrix::Mapping(matrix),
        Value::Bool(_)
        | Value::Number(_)
        | Value::String(_)
        | Value::Null
        | Value::Sequence(_)
        | Value::Tagged(_) => ResolvedRootMatrix::NonMapping,
    }
}
