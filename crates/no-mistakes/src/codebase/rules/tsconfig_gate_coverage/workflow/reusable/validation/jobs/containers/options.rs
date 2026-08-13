use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    resolve_static_interpolations, EnvironmentState, InputState,
};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::{
    reduce_context_free_interpolations, ContextFreeInterpolation,
};

use super::ContainerKind;

pub(super) fn shape_valid(value: &str, kind: ContainerKind) -> bool {
    super::valid_container_value(value)
        && match reduce_context_free_interpolations(value) {
            ContextFreeInterpolation::Static(value) => supported(&value, kind),
            ContextFreeInterpolation::Dynamic => true,
            ContextFreeInterpolation::Invalid => false,
        }
}

pub(super) fn valid_for_inputs(
    value: Option<&str>,
    kind: ContainerKind,
    inputs: &InputState,
    environment: &EnvironmentState,
) -> bool {
    value.is_none_or(|value| {
        resolve_static_interpolations(value, inputs, environment)
            .is_none_or(|value| supported(&value, kind))
    })
}

fn supported(value: &str, kind: ContainerKind) -> bool {
    let Some(tokens) = tokens(value) else {
        return false;
    };
    !tokens.is_empty()
        && tokens.iter().all(|token| {
            !unsupported_flag(token, "--network")
                && (!matches!(kind, ContainerKind::Job) || !unsupported_flag(token, "--entrypoint"))
        })
}

fn unsupported_flag(token: &str, flag: &str) -> bool {
    token == flag
        || token
            .strip_prefix(flag)
            .is_some_and(|suffix| suffix.starts_with('='))
}

fn tokens(value: &str) -> Option<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaped = false;
    for character in value.chars() {
        if escaped {
            current.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' && quote != Some('\'') {
            escaped = true;
            continue;
        }
        if matches!(character, '\'' | '"') {
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            } else {
                current.push(character);
            }
            continue;
        }
        if character.is_ascii_whitespace() && quote.is_none() {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
        } else {
            current.push(character);
        }
    }
    if escaped || quote.is_some() {
        return None;
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    Some(tokens)
}

#[cfg(test)]
mod tests;
