use super::{InputState, StaticValue};

pub(crate) fn complete_expression_static_string_value(
    value: &str,
    inputs: &InputState,
) -> Option<StaticValue> {
    let expression = value.trim().strip_prefix("${{")?.strip_suffix("}}")?.trim();
    super::super::expressions::complete_literal_expression_value(value)
        .map(super::static_yaml_value)
        .or_else(|| static_expression_value(expression, inputs))
}

fn static_expression_value(expression: &str, inputs: &InputState) -> Option<StaticValue> {
    if let Some(name) = super::resolution::input_name(expression) {
        return Some(
            inputs
                .get(&name.to_lowercase())
                .cloned()
                .unwrap_or_else(|| StaticValue::String(String::new())),
        );
    }
    if let Some(call) = super::super::expressions::condition_function_call(expression) {
        if call.function == super::super::expressions::Function::FromJson
            && call.arguments.len() == 1
        {
            return static_from_json(call.arguments[0], inputs);
        }
    }
    super::resolution::condition_input_value(
        expression,
        inputs,
        &super::EnvironmentState::default(),
    )
}

fn static_from_json(argument: &str, inputs: &InputState) -> Option<StaticValue> {
    let value = static_expression_value(argument, inputs)?;
    let encoded = match value {
        StaticValue::Unknown => return None,
        StaticValue::Sequence(_) | StaticValue::NonStringable => {
            return Some(StaticValue::NonStringable)
        }
        value => value.function_string()?,
    };
    Some(
        serde_json::from_str::<serde_json::Value>(&encoded)
            .ok()
            .and_then(|value| serde_yaml::to_value(value).ok())
            .map(super::static_yaml_value)
            .unwrap_or(StaticValue::NonStringable),
    )
}
