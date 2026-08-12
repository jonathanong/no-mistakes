use super::icons::branding_icon_valid;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::{
    condition_expression_contexts_available, interpolated_expression_contexts_available,
};
use serde_yaml::{Mapping, Value};
use std::collections::BTreeSet;

#[cfg(test)]
mod tests;
const COMPOSITE_OUTPUT_CONTEXTS: &[&str] = &["github", "inputs", "steps", "runner", "env", "vars"];
const ACTION_PRE_IF_CONTEXTS: &[&str] = &[
    "github", "needs", "strategy", "matrix", "job", "runner", "env", "vars", "inputs",
];
const DOCKER_ARGUMENT_CONTEXTS: &[&str] = &["github", "inputs"];
pub(super) fn action_inputs_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value.as_mapping().is_some_and(|mapping| {
            let mut names = BTreeSet::new();
            mapping.iter().all(|(name, metadata)| {
                let Some(name) = name.as_str() else {
                    return false;
                };
                let Some(metadata) = metadata.as_mapping() else {
                    return false;
                };
                action_identifier(name)
                    && names.insert(name.to_ascii_lowercase())
                    && only_keys(
                        metadata,
                        &["description", "required", "default", "deprecationMessage"],
                    )
                    && nonempty_string(metadata.get("description"))
                    && metadata
                        .get("required")
                        .is_none_or(|value| matches!(value, Value::Bool(_)))
                    && metadata
                        .get("default")
                        .is_none_or(action_input_default_valid)
                    && metadata
                        .get("deprecationMessage")
                        .is_none_or(|value| matches!(value, Value::String(_)))
            })
        })
    })
}

fn action_input_default_valid(value: &Value) -> bool {
    matches!(
        value,
        Value::Bool(_) | Value::Number(_) | Value::String(_) | Value::Null
    )
}

pub(super) fn outputs_valid(value: Option<&Value>, composite: bool) -> bool {
    value.is_none_or(|value| {
        value.as_mapping().is_some_and(|mapping| {
            let mut names = BTreeSet::new();
            mapping.iter().all(|(name, metadata)| {
                let Some(name) = name.as_str() else {
                    return false;
                };
                let Some(metadata) = metadata.as_mapping() else {
                    return false;
                };
                action_identifier(name)
                    && names.insert(name.to_ascii_lowercase())
                    && only_keys(metadata, &["description", "value"])
                    && nonempty_string(metadata.get("description"))
                    && if composite {
                        metadata
                            .get("value")
                            .and_then(Value::as_str)
                            .is_some_and(|value| {
                                interpolated_expression_contexts_available(
                                    value,
                                    COMPOSITE_OUTPUT_CONTEXTS,
                                )
                            })
                    } else {
                        metadata.get("value").is_none()
                    }
            })
        })
    })
}

pub(super) fn branding_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        let Some(branding) = value.as_mapping() else {
            return false;
        };
        only_keys(branding, &["icon", "color"])
            && branding
                .get("icon")
                .and_then(Value::as_str)
                .is_some_and(branding_icon_valid)
            && branding
                .get("color")
                .and_then(Value::as_str)
                .is_some_and(|color| {
                    matches!(
                        color,
                        "white"
                            | "black"
                            | "yellow"
                            | "blue"
                            | "green"
                            | "orange"
                            | "red"
                            | "purple"
                            | "gray-dark"
                    )
                })
    })
}

pub(super) fn runs_shape_valid(runs: &Mapping, using: &str) -> bool {
    let keys = match using {
        "composite" => &["using", "steps"][..],
        "node" => &["using", "main", "post", "post-if"][..],
        "docker" => &[
            "using",
            "image",
            "args",
            "env",
            "entrypoint",
            "pre-entrypoint",
            "post-entrypoint",
            "pre-if",
            "post-if",
        ][..],
        _ => return false,
    };
    if !only_keys(runs, keys)
        || !nonempty_string(runs.get("using"))
        || !runs
            .get("post")
            .is_none_or(|value| nonempty_string(Some(value)))
        || !runs.get("pre-if").is_none_or(pre_if_valid)
        || !runs.get("post-if").is_none_or(pre_if_valid)
        || !runs
            .get("pre-entrypoint")
            .is_none_or(|value| nonempty_string(Some(value)))
        || !runs
            .get("post-entrypoint")
            .is_none_or(|value| nonempty_string(Some(value)))
        || !runs
            .get("entrypoint")
            .is_none_or(|value| nonempty_string(Some(value)))
    {
        return false;
    }
    if let Some(args) = runs.get("args") {
        if !args.as_sequence().is_some_and(|args| {
            args.iter().all(|value| {
                value.as_str().is_some_and(|value| {
                    interpolated_expression_contexts_available(value, DOCKER_ARGUMENT_CONTEXTS)
                })
            })
        }) {
            return false;
        }
    }
    if let Some(env) = runs.get("env") {
        if !env.as_mapping().is_some_and(|env| {
            env.iter().all(|(name, value)| {
                name.as_str().is_some()
                    && value.as_str().is_some_and(|value| {
                        interpolated_expression_contexts_available(value, DOCKER_ARGUMENT_CONTEXTS)
                    })
            })
        }) {
            return false;
        }
    }
    true
}

fn pre_if_valid(value: &Value) -> bool {
    value.as_str().is_some_and(|value| {
        condition_expression_contexts_available(value, ACTION_PRE_IF_CONTEXTS, false)
    })
}

pub(super) fn only_keys(mapping: &Mapping, keys: &[&str]) -> bool {
    mapping
        .keys()
        .all(|key| key.as_str().is_some_and(|key| keys.contains(&key)))
}

pub(super) fn nonempty_string(value: Option<&Value>) -> bool {
    value
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty())
}

pub(super) fn action_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && characters
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
}
