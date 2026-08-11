use super::{EnvironmentState, InputState, StaticValue};

pub(crate) fn complete_expression_static_string_value(
    value: &str,
    inputs: &InputState,
) -> Option<StaticValue> {
    complete_expression_static_value_with_environment(value, inputs, &EnvironmentState::default())
}

pub(crate) fn complete_expression_static_value_with_environment(
    value: &str,
    inputs: &InputState,
    environment: &EnvironmentState,
) -> Option<StaticValue> {
    let expression = value.trim().strip_prefix("${{")?.strip_suffix("}}")?.trim();
    super::super::expressions::complete_literal_expression_value(value)
        .map(super::static_yaml_value)
        .or_else(|| static_expression_value(expression, inputs, environment))
}

fn static_expression_value(
    expression: &str,
    inputs: &InputState,
    environment: &EnvironmentState,
) -> Option<StaticValue> {
    if super::resolution::github_event_name(expression) {
        return super::inputs::event_name_value(inputs);
    }
    if super::resolution::github_event_action(expression) {
        return super::inputs::event_action_value(inputs);
    }
    if super::resolution::github_ref(expression) {
        return inputs.get(super::inputs::REF_KEY).cloned();
    }
    if super::resolution::github_ref_name(expression) {
        return super::inputs::event_ref_name_value(inputs);
    }
    if super::resolution::github_base_ref(expression) {
        return super::inputs::event_base_ref_value(inputs);
    }
    if let Some(name) = super::resolution::input_name(expression) {
        return Some(
            inputs
                .get(&name.to_lowercase())
                .cloned()
                .unwrap_or_else(|| StaticValue::String(String::new())),
        );
    }
    if let Some(value) = static_from_json_expression(expression, inputs, environment) {
        return Some(value);
    }
    super::resolution::condition_input_value(expression, inputs, environment).or_else(|| {
        super::condition_values::condition_value(
            expression,
            inputs,
            environment,
            super::ConditionStatus::SUCCESS,
        )
    })
}

pub(super) fn static_from_json_expression(
    expression: &str,
    inputs: &InputState,
    environment: &EnvironmentState,
) -> Option<StaticValue> {
    let call = super::super::expressions::condition_function_call(expression)?;
    (call.function == super::super::expressions::Function::FromJson && call.arguments.len() == 1)
        .then(|| static_from_json(call.arguments[0], inputs, environment))?
}

fn static_from_json(
    argument: &str,
    inputs: &InputState,
    environment: &EnvironmentState,
) -> Option<StaticValue> {
    let value = static_expression_value(argument, inputs, environment)?;
    let encoded = match value {
        StaticValue::Unknown => return None,
        StaticValue::Sequence(_) | StaticValue::Mapping | StaticValue::NonStringable => {
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
