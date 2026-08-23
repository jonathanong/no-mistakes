use super::{function_argument_value, ConditionStatus, EnvironmentState, InputState, StaticValue};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::{
    condition_function_call, Function,
};

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions) fn static_join_value(
    expression: &str,
    inputs: &InputState,
    environment: &EnvironmentState,
    status: impl Into<ConditionStatus>,
) -> Option<StaticValue> {
    let status = status.into();
    let call = condition_function_call(expression)?;
    (call.function == Function::Join).then_some(())?;
    let value = function_argument_value(call.arguments.first()?, inputs, environment, status)?;
    if matches!(value, StaticValue::Invalid) {
        return Some(StaticValue::Invalid);
    }
    let separator = match call.arguments.get(1) {
        Some(argument) => {
            let separator = function_argument_value(argument, inputs, environment, status)?;
            if matches!(separator, StaticValue::Invalid) {
                return Some(StaticValue::Invalid);
            }
            separator.format_string()?
        }
        None => ",".to_string(),
    };
    match value {
        StaticValue::Sequence(values) if values.len() > 1 => {
            if values
                .iter()
                .any(|value| matches!(value, StaticValue::Invalid))
            {
                return Some(StaticValue::Invalid);
            }
            values
                .iter()
                .map(StaticValue::format_string)
                .collect::<Option<Vec<_>>>()
                .map(|values| StaticValue::String(values.join(&separator)))
        }
        StaticValue::Sequence(values) => Some(StaticValue::String(match values.first() {
            Some(StaticValue::Invalid) => return Some(StaticValue::Invalid),
            Some(value) => value.format_string()?,
            None => String::new(),
        })),
        value => value.format_string().map(StaticValue::String),
    }
}
