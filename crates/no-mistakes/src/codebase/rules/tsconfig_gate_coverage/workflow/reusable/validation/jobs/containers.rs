use serde_yaml::{Mapping, Value};

use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::valid_identifier;

use super::super::super::super::expressions::interpolated_expression_contexts_available;
use super::super::super::super::expressions::interpolation::opaque_interpolated_expression_form;
use super::values::{only_keys, scalar_mapping_valid};

mod images;

const CONTAINER_CONTEXTS: &[&str] = &["github", "needs", "strategy", "matrix", "vars", "inputs"];
const CONTAINER_ENV_CONTEXTS: &[&str] = &[
    "github", "needs", "strategy", "matrix", "job", "runner", "env", "vars", "secrets", "inputs",
];
const CONTAINER_CREDENTIAL_CONTEXTS: &[&str] = &[
    "github", "needs", "strategy", "matrix", "env", "vars", "secrets", "inputs",
];
const DYNAMIC_VOLUME_EXPRESSION: &str = "\u{FDD1}";

pub(super) fn container_shape_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value.as_str().is_some_and(valid_container_image)
            || value
                .as_mapping()
                .is_some_and(container_mapping_shape_valid)
    })
}

pub(super) fn services_shape_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value.as_mapping().is_some_and(|services| {
            !services.is_empty()
                && services.iter().all(|(name, service)| {
                    name.as_str().is_some_and(valid_identifier)
                        && service
                            .as_mapping()
                            .is_some_and(container_mapping_shape_valid)
                })
        })
    })
}

fn container_mapping_shape_valid(container: &Mapping) -> bool {
    only_keys(
        container,
        &["image", "credentials", "env", "ports", "volumes", "options"],
    ) && container
        .get("image")
        .and_then(Value::as_str)
        .is_some_and(valid_container_image)
        && credentials_shape_valid(container.get("credentials"))
        && scalar_mapping_valid(container.get("env"), CONTAINER_ENV_CONTEXTS, false)
        && scalar_sequence_contexts_valid(container.get("ports"), CONTAINER_CONTEXTS)
        && super::ports::port_sequence_valid(container.get("ports"))
        && volume_sequence_valid(container.get("volumes"), CONTAINER_CONTEXTS)
        && container
            .get("options")
            .is_none_or(|value| value.as_str().is_some_and(valid_container_value))
}

fn credentials_shape_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value.as_mapping().is_some_and(|credentials| {
            only_keys(credentials, &["username", "password"])
                && credentials
                    .get("username")
                    .and_then(Value::as_str)
                    .is_some_and(valid_container_credential)
                && credentials
                    .get("password")
                    .and_then(Value::as_str)
                    .is_some_and(valid_container_credential)
        })
    })
}

fn valid_container_credential(value: &str) -> bool {
    valid_contextual_value(value, CONTAINER_CREDENTIAL_CONTEXTS)
}

fn volume_sequence_valid(value: Option<&Value>, allowed_contexts: &[&str]) -> bool {
    value.is_none_or(|value| {
        value.as_sequence().is_some_and(|items| {
            items.iter().all(|item| {
                item.as_str().is_some_and(|value| {
                    valid_contextual_value(value, allowed_contexts) && valid_volume(value)
                })
            })
        })
    })
}

fn valid_volume(value: &str) -> bool {
    let Some(value) = opaque_interpolated_expression_form(value, DYNAMIC_VOLUME_EXPRESSION) else {
        return false;
    };
    if value == DYNAMIC_VOLUME_EXPRESSION {
        return true;
    }
    if !value.contains(':') {
        return value.starts_with('/');
    }
    let mut parts = value.split(':');
    let Some(source) = parts.next() else {
        return false;
    };
    let Some(destination) = parts.next() else {
        return false;
    };
    parts.next().is_none() && volume_source_valid(source) && volume_destination_valid(destination)
}

fn volume_source_valid(value: &str) -> bool {
    value.starts_with('/')
        || value.starts_with(DYNAMIC_VOLUME_EXPRESSION)
        || (!value.is_empty()
            && value
                .replace(DYNAMIC_VOLUME_EXPRESSION, "dynamic")
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-')))
}

fn volume_destination_valid(value: &str) -> bool {
    value.starts_with('/') || value.starts_with(DYNAMIC_VOLUME_EXPRESSION)
}

fn scalar_sequence_contexts_valid(value: Option<&Value>, allowed_contexts: &[&str]) -> bool {
    value.is_none_or(|value| {
        value.as_sequence().is_some_and(|items| {
            items.iter().all(|item| {
                !item.is_string()
                    || item.as_str().is_some_and(|value| {
                        interpolated_expression_contexts_available(value, allowed_contexts)
                    })
            })
        })
    })
}

fn valid_container_value(value: &str) -> bool {
    valid_contextual_value(value, CONTAINER_CONTEXTS)
}

fn valid_container_image(value: &str) -> bool {
    valid_contextual_value(value, CONTAINER_CONTEXTS) && images::valid(value)
}

fn valid_contextual_value(value: &str, allowed_contexts: &[&str]) -> bool {
    !value.is_empty() && interpolated_expression_contexts_available(value, allowed_contexts)
}
