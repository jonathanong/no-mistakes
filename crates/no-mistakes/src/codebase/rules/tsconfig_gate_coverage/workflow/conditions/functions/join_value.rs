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
    match value {
        StaticValue::Sequence(values) if values.len() > 1 => {
            let separator = match call.arguments.get(1) {
                Some(argument) => function_argument_value(argument, inputs, environment, status)?
                    .format_string()?,
                None => ",".to_string(),
            };
            values
                .iter()
                .map(StaticValue::format_string)
                .collect::<Option<Vec<_>>>()
                .map(|values| StaticValue::String(values.join(&separator)))
        }
        StaticValue::Sequence(values) => Some(StaticValue::String(match values.first() {
            Some(value) => value.format_string()?,
            None => String::new(),
        })),
        value => value.format_string().map(StaticValue::String),
    }
}
