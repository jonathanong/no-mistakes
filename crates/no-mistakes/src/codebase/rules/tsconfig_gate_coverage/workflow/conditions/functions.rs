use super::{
    condition_values::condition_value, input_value::comparison_literal, resolution::input_name,
    ConditionStatus, EnvironmentState, InputState, StaticBool, StaticValue,
};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::{
    condition_function_call, Function,
};

mod format_value;
pub(super) use format_value::static_format_value;
mod join_value;
pub(super) use join_value::static_join_value;

pub(super) fn static_function_bool(
    expression: &str,
    inputs: &InputState,
    environment: &EnvironmentState,
    status: ConditionStatus,
) -> Option<StaticBool> {
    let call = condition_function_call(expression)?;
    if call.function == Function::Case {
        return static_case_value(expression, inputs, environment, status)
            .map(StaticValue::truthiness);
    }
    if call.function == Function::Format {
        return static_format_value(expression, inputs, environment, status)
            .map(|value| value.truthiness());
    }
    if call.function == Function::Join {
        return static_join_value(expression, inputs, environment, status)
            .map(|value| value.truthiness());
    }
    if call.arguments.len() != 2 {
        return None;
    }
    let search = function_argument_value(call.arguments[0], inputs, environment, status)?;
    if matches!(search, StaticValue::Invalid) {
        return Some(StaticBool::Invalid);
    }
    let item = function_argument_value(call.arguments[1], inputs, environment, status)?;
    if matches!(item, StaticValue::Invalid) {
        return Some(StaticBool::Invalid);
    }
    let item = item.function_string()?;
    let matched = match call.function {
        Function::Contains => contains_static_value(&search, &item)?,
        Function::StartsWith => starts_with_ignore_ascii_case(&search.function_string()?, &item)?,
        Function::EndsWith => ends_with_ignore_ascii_case(&search.function_string()?, &item)?,
        _ => return None,
    };
    Some(StaticBool::from(matched))
}

fn contains_static_value(search: &StaticValue, item: &str) -> Option<bool> {
    let StaticValue::Sequence(values) = search else {
        return contains_ignore_ascii_case(&search.function_string()?, item);
    };
    let mut unknown = false;
    for value in values {
        if matches!(value, StaticValue::Mapping | StaticValue::NonStringable) {
            continue;
        }
        match value
            .function_string()
            .and_then(|value| string_equals_ignore_ascii_case(&value, item))
        {
            Some(true) => return Some(true),
            Some(false) => {}
            None => unknown = true,
        }
    }
    (!unknown).then_some(false)
}

pub(super) fn static_case_value(
    expression: &str,
    inputs: &InputState,
    environment: &EnvironmentState,
    status: impl Into<ConditionStatus>,
) -> Option<StaticValue> {
    let status = status.into();
    let call = condition_function_call(expression)?;
    (call.function == Function::Case).then_some(())?;
    for index in (0..call.arguments.len() - 1).step_by(2) {
        match function_argument_value(call.arguments[index], inputs, environment, status)?
            .truthiness()
        {
            StaticBool::False => continue,
            StaticBool::True | StaticBool::TruthyNonBoolean => {
                return function_argument_value(
                    call.arguments[index + 1],
                    inputs,
                    environment,
                    status,
                );
            }
            StaticBool::Invalid => return Some(StaticValue::Invalid),
            StaticBool::Unknown => return None,
        }
    }
    function_argument_value(call.arguments.last()?, inputs, environment, status)
}

fn function_argument_value(
    expression: &str,
    inputs: &InputState,
    environment: &EnvironmentState,
    status: ConditionStatus,
) -> Option<StaticValue> {
    if let Some(name) = input_name(expression) {
        // The boolean/equality condition path intentionally models a missing
        // input as `false`; GitHub's string functions instead coerce a missing
        // property through null to the empty string.
        return Some(
            inputs
                .get(&name.to_lowercase())
                .cloned()
                .unwrap_or_else(|| StaticValue::String(String::new())),
        );
    }
    condition_value(expression, inputs, environment, status)
        .or_else(|| comparison_literal(expression))
}

// GitHub's documented functions compare case-insensitively. This analyzer only
// models ASCII strings, whose case mapping is unambiguous; non-ASCII literals
// remain unknown so they cannot create unsupported coverage credit.
fn contains_ignore_ascii_case(search: &str, item: &str) -> Option<bool> {
    search.is_ascii().then_some(())?;
    item.is_ascii().then_some(())?;
    if item.is_empty() {
        return Some(true);
    }
    Some(
        search
            .as_bytes()
            .windows(item.len())
            .any(|candidate| candidate.eq_ignore_ascii_case(item.as_bytes())),
    )
}

fn string_equals_ignore_ascii_case(left: &str, right: &str) -> Option<bool> {
    (left.is_ascii() && right.is_ascii()).then(|| left.eq_ignore_ascii_case(right))
}

fn starts_with_ignore_ascii_case(search: &str, item: &str) -> Option<bool> {
    search.is_ascii().then_some(())?;
    item.is_ascii().then_some(())?;
    Some(
        search
            .as_bytes()
            .get(..item.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(item.as_bytes())),
    )
}

fn ends_with_ignore_ascii_case(search: &str, item: &str) -> Option<bool> {
    search.is_ascii().then_some(())?;
    item.is_ascii().then_some(())?;
    Some(
        search
            .as_bytes()
            .get(search.len().saturating_sub(item.len())..)
            .is_some_and(|suffix| suffix.eq_ignore_ascii_case(item.as_bytes())),
    )
}
