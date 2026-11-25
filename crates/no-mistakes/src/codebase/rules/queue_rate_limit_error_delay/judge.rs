use super::options::Target;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::module_facts::arguments::{ArgumentValue, ArgumentValues};
use serde_json::Value;

pub(super) type AbsentSlot = (usize, Option<String>);

pub(super) struct CallDelayArgs<'a> {
    pub(super) values: Option<&'a ArgumentValues>,
    pub(super) absent: Option<&'a [AbsentSlot]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum DelayVerdict {
    Valid,
    Missing(String),
    Unanalyzable(String),
}

pub(super) fn call_arguments(facts: &TsFileFacts, offset: u32) -> CallDelayArgs<'_> {
    if facts.operational_error.is_some() || facts.parse_error.is_some() || facts.fatal_parse_error {
        return CallDelayArgs {
            values: None,
            absent: None,
        };
    }
    let Some(module) = facts.module_bindings.as_deref() else {
        return CallDelayArgs {
            values: None,
            absent: None,
        };
    };
    if !module
        .diagnostics
        .iter()
        .all(crate::codebase::ts_source::module_facts::argument_safe_diagnostic)
    {
        return CallDelayArgs {
            values: None,
            absent: None,
        };
    }
    let values = module
        .call_arguments
        .as_deref()
        .and_then(|m| m.get(&offset));
    let absent = module
        .path_provenance
        .as_deref()
        .and_then(|p| p.absent_or_non_finite_args.get(&offset))
        .map(Vec::as_slice);
    CallDelayArgs { values, absent }
}

pub(super) fn judge_arguments(
    arguments: Option<&ArgumentValues>,
    absent_slots: Option<&[AbsentSlot]>,
    target: &Target,
) -> DelayVerdict {
    let Some(arguments) = arguments else {
        return DelayVerdict::Unanalyzable(
            "call arguments could not be statically analyzed".to_string(),
        );
    };
    let pos = target.delay_argument;
    if arguments.spread_from.is_some_and(|start| pos >= start) {
        return DelayVerdict::Unanalyzable(format!(
            "spread argument at or before argument {pos} hides whether a delay is supplied"
        ));
    }
    let Some(value) = arguments.values.get(pos) else {
        return DelayVerdict::Missing(missing_delay_detail(target));
    };
    if has_absent_slot(absent_slots, pos, None) {
        return DelayVerdict::Missing(match &target.delay_property {
            Some(prop) => format!(
                "options argument {pos} is undefined instead of providing a positive `{prop}` delay"
            ),
            None => format!("delay argument {pos} is undefined or non-finite"),
        });
    }
    match &target.delay_property {
        None => judge_scalar_delay(value, pos),
        Some(prop) => judge_property_delay(value, absent_slots, pos, prop),
    }
}

fn judge_scalar_delay(value: &ArgumentValue, pos: usize) -> DelayVerdict {
    match value {
        ArgumentValue::Scalar(Value::Number(num))
            if num.as_f64().is_some_and(|n| n.is_finite() && n > 0.0) =>
        {
            DelayVerdict::Valid
        }
        ArgumentValue::Unknown => DelayVerdict::Valid,
        ArgumentValue::Scalar(_) | ArgumentValue::Array(_) | ArgumentValue::Object(_) => {
            DelayVerdict::Missing(format!("delay argument {pos} must be a positive duration"))
        }
    }
}

fn judge_property_delay(
    value: &ArgumentValue,
    absent_slots: Option<&[AbsentSlot]>,
    pos: usize,
    prop: &str,
) -> DelayVerdict {
    match value {
        ArgumentValue::Object(map) => {
            let Some(prop_value) = map.get(prop) else {
                return DelayVerdict::Missing(format!(
                    "options argument {pos} omits required `{prop}` delay"
                ));
            };
            if has_absent_slot(absent_slots, pos, Some(prop)) {
                return DelayVerdict::Missing(format!(
                    "options property `{prop}` is undefined or non-finite"
                ));
            }
            match prop_value {
                ArgumentValue::Scalar(Value::Number(num))
                    if num.as_f64().is_some_and(|n| n.is_finite() && n > 0.0) =>
                {
                    DelayVerdict::Valid
                }
                ArgumentValue::Unknown => DelayVerdict::Valid,
                ArgumentValue::Scalar(_) | ArgumentValue::Array(_) | ArgumentValue::Object(_) => {
                    DelayVerdict::Missing(format!(
                        "options property `{prop}` must be a positive duration"
                    ))
                }
            }
        }
        ArgumentValue::Unknown => DelayVerdict::Unanalyzable(format!(
            "options argument {pos} is not a closed object literal, so `{prop}` could not be verified"
        )),
        ArgumentValue::Scalar(_) | ArgumentValue::Array(_) => DelayVerdict::Missing(format!(
            "options argument {pos} must be an object supplying a positive `{prop}` delay"
        )),
    }
}

fn has_absent_slot(absent_slots: Option<&[AbsentSlot]>, pos: usize, prop: Option<&str>) -> bool {
    absent_slots.is_some_and(|slots| {
        slots
            .iter()
            .any(|(index, key)| *index == pos && key.as_deref() == prop)
    })
}

fn missing_delay_detail(target: &Target) -> String {
    match &target.delay_property {
        Some(prop) => format!(
            "omits options argument {} with required `{prop}` delay",
            target.delay_argument
        ),
        None => format!("omits delay argument at position {}", target.delay_argument),
    }
}
