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
    crate::codebase::shell_words::split_quoted_words(
        value,
        crate::codebase::shell_words::Whitespace::Ascii,
    )
}

#[cfg(test)]
mod tests;
