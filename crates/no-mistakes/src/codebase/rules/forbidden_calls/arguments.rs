use super::config::{ArgumentConditions, ArgumentPredicate, Literal};
use crate::codebase::ts_source::module_facts::arguments::{ArgumentValue, ArgumentValues};

pub(crate) fn for_call(
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
    offset: u32,
) -> Option<&ArgumentValues> {
    // Recovered syntax and failed semantic ownership cannot prove a predicate.
    if facts.operational_error.is_some() || facts.parse_error.is_some() || facts.fatal_parse_error {
        return None;
    }
    let module = facts.module_bindings.as_deref()?;
    if !module.diagnostics.is_empty() {
        return None;
    }
    module.call_arguments.as_deref()?.get(&offset)
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Match {
    Yes,
    No,
    Unknown,
}

pub(super) fn matches(
    conditions: &ArgumentConditions,
    arguments: Option<&ArgumentValues>,
) -> Match {
    let Some(arguments) = arguments else {
        return Match::Unknown;
    };
    let mut unknown = false;
    for predicate in &conditions.all {
        match one(predicate, arguments) {
            Match::No => return Match::No,
            Match::Unknown => unknown = true,
            Match::Yes => {}
        }
    }
    if unknown {
        Match::Unknown
    } else {
        Match::Yes
    }
}

fn one(predicate: &ArgumentPredicate, arguments: &ArgumentValues) -> Match {
    let (position, property) = predicate.selector();
    if arguments.spread_from.is_some_and(|start| position >= start) {
        return Match::Unknown;
    }
    let Some(mut value) = arguments.values.get(position) else {
        return Match::No;
    };
    if let Some(property) = property {
        match value {
            ArgumentValue::Object(values) => {
                let Some(selected) = values.get(property) else {
                    return Match::No;
                };
                value = selected;
            }
            ArgumentValue::Unknown => return Match::Unknown,
            _ => return Match::No,
        }
    }
    match predicate {
        ArgumentPredicate::Equals(p) => literal(&p.equals, value),
        ArgumentPredicate::Includes(p) => includes(&p.includes, value),
        ArgumentPredicate::Excludes(p) => match includes(&p.excludes, value) {
            Match::Yes => Match::No,
            Match::No if matches!(value, ArgumentValue::Array(_)) => Match::Yes,
            result => result,
        },
    }
}

fn literal(expected: &Literal, value: &ArgumentValue) -> Match {
    let actual = match value {
        ArgumentValue::Unknown => return Match::Unknown,
        ArgumentValue::Scalar(actual) => actual,
        _ => return Match::No,
    };
    let equal = match expected {
        Literal::String(value) => actual.as_str() == Some(value),
        Literal::Boolean(value) => actual.as_bool() == Some(*value),
        Literal::Number(value) => actual.as_f64() == Some(*value),
        Literal::Null(()) => actual.is_null(),
    };
    if equal {
        Match::Yes
    } else {
        Match::No
    }
}

fn includes(expected: &Literal, value: &ArgumentValue) -> Match {
    let values = match value {
        ArgumentValue::Array(values) => values,
        ArgumentValue::Unknown => return Match::Unknown,
        _ => return Match::No,
    };
    let mut unknown = false;
    for value in values {
        match literal(expected, value) {
            Match::Yes => return Match::Yes,
            Match::Unknown => unknown = true,
            Match::No => {}
        }
    }
    if unknown {
        Match::Unknown
    } else {
        Match::No
    }
}

#[cfg(test)]
#[path = "argument_match_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "argument_confidence_tests.rs"]
mod confidence_tests;
