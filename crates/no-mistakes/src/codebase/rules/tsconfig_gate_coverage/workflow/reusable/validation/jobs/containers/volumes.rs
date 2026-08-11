use serde_yaml::Value;

use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    resolve_static_interpolations, EnvironmentState, InputState,
};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::interpolation::opaque_interpolated_expression_form;

const DYNAMIC_EXPRESSION: &str = "\u{FDD1}";

pub(super) fn shape_valid(value: Option<&Value>, allowed_contexts: &[&str]) -> bool {
    value.is_none_or(|value| {
        value.as_sequence().is_some_and(|items| {
            items.iter().all(|item| {
                item.as_str().is_some_and(|value| {
                    super::valid_contextual_value(value, allowed_contexts) && volume_valid(value)
                })
            })
        })
    })
}

pub(super) fn valid_for_inputs(
    value: Option<&Value>,
    inputs: &InputState,
    environment: &EnvironmentState,
) -> bool {
    value.is_none_or(|value| {
        value.as_sequence().is_some_and(|items| {
            items.iter().all(|item| {
                item.as_str().is_some_and(|value| {
                    resolve_static_interpolations(value, inputs, environment)
                        .map_or_else(|| volume_valid(value), |value| volume_valid(&value))
                })
            })
        })
    })
}

fn volume_valid(value: &str) -> bool {
    let Some(value) = opaque_interpolated_expression_form(value, DYNAMIC_EXPRESSION) else {
        return false;
    };
    if value == DYNAMIC_EXPRESSION {
        return true;
    }
    if !value.contains(':') {
        return value.starts_with('/');
    }
    let mut parts = value.split(':');
    let source = parts.next().expect("split always returns a source");
    let destination = parts.next().expect("value contains a separator");
    parts.next().is_none() && source_valid(source) && destination_valid(destination)
}

fn source_valid(value: &str) -> bool {
    if value.starts_with('/') || value.starts_with(DYNAMIC_EXPRESSION) {
        return true;
    }
    let normalized = value.replace(DYNAMIC_EXPRESSION, "dynamic");
    normalized
        .bytes()
        .next()
        .is_some_and(|byte| byte.is_ascii_alphanumeric())
        && normalized
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-'))
}

fn destination_valid(value: &str) -> bool {
    value.starts_with('/') || value.starts_with(DYNAMIC_EXPRESSION)
}

#[cfg(test)]
mod tests;
