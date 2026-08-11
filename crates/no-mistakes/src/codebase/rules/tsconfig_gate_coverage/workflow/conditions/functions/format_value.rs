use super::{function_argument_value, ConditionStatus, EnvironmentState, InputState, StaticValue};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::{
    condition_function_call, Function,
};

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions) fn static_format_value(
    expression: &str,
    inputs: &InputState,
    environment: &EnvironmentState,
    status: impl Into<ConditionStatus>,
) -> Option<StaticValue> {
    let status = status.into();
    let call = condition_function_call(expression)?;
    (call.function == Function::Format).then_some(())?;
    let format = function_argument_value(call.arguments.first()?, inputs, environment, status)?
        .format_string()?;
    let replacements = call.arguments[1..]
        .iter()
        .map(|argument| {
            function_argument_value(argument, inputs, environment, status)
                .and_then(|value| value.format_string())
        })
        .collect::<Option<Vec<_>>>()?;
    format_github_string(&format, &replacements).map(StaticValue::String)
}

/// GitHub Actions `format` uses zero-based `{N}` placeholders. Literal braces
/// are escaped by doubling them, so malformed braces and missing replacements
/// stay unknown instead of producing coverage credit.
fn format_github_string(format: &str, replacements: &[String]) -> Option<String> {
    let mut rendered = String::with_capacity(format.len());
    let mut characters = format.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '{' if characters.next_if_eq(&'{').is_some() => rendered.push('{'),
            '}' if characters.next_if_eq(&'}').is_some() => rendered.push('}'),
            '{' => {
                let mut index = String::new();
                let mut closed = false;
                for character in characters.by_ref() {
                    if character == '}' {
                        closed = true;
                        break;
                    }
                    character.is_ascii_digit().then_some(())?;
                    index.push(character);
                }
                closed.then_some(())?;
                (!index.is_empty()).then_some(())?;
                let index = index.parse::<usize>().ok()?;
                rendered.push_str(replacements.get(index)?);
            }
            '}' => return None,
            character => rendered.push(character),
        }
    }
    Some(rendered)
}
