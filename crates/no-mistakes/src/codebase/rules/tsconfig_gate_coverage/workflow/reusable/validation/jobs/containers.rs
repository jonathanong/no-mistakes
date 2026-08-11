use serde_yaml::{Mapping, Value};

use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::valid_identifier;

use super::super::super::super::expressions::{
    interpolated_expression_contexts_available, reduce_context_free_interpolations,
    ContextFreeInterpolation,
};
use super::values::{only_keys, scalar_mapping_valid};

mod configuration;
mod images;
mod options;
mod volumes;

pub(crate) use configuration::container_configuration_valid_for_inputs;

#[derive(Clone, Copy)]
enum ContainerKind {
    Job,
    Service,
}

const CONTAINER_CONTEXTS: &[&str] = &["github", "needs", "strategy", "matrix", "vars", "inputs"];
const CONTAINER_ENV_CONTEXTS: &[&str] = &[
    "github", "needs", "strategy", "matrix", "job", "runner", "env", "vars", "secrets", "inputs",
];
const CONTAINER_CREDENTIAL_CONTEXTS: &[&str] = &[
    "github", "needs", "strategy", "matrix", "env", "vars", "secrets", "inputs",
];

pub(super) fn container_shape_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value.as_str().is_some_and(valid_container_image)
            || value.as_mapping().is_some_and(|container| {
                container_mapping_shape_valid(container, ContainerKind::Job)
            })
    })
}

pub(super) fn services_shape_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value.as_mapping().is_some_and(|services| {
            !services.is_empty()
                && services.iter().all(|(name, service)| {
                    name.as_str().is_some_and(valid_identifier)
                        && service.as_mapping().is_some_and(|service| {
                            container_mapping_shape_valid(service, ContainerKind::Service)
                        })
                })
        })
    })
}

fn container_mapping_shape_valid(container: &Mapping, kind: ContainerKind) -> bool {
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
        && volumes::shape_valid(container.get("volumes"), CONTAINER_CONTEXTS)
        && container.get("options").is_none_or(|value| {
            value
                .as_str()
                .is_some_and(|value| options::shape_valid(value, kind))
        })
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

pub(super) fn valid_container_image(value: &str) -> bool {
    !value.is_empty()
        && interpolated_expression_contexts_available(value, CONTAINER_CONTEXTS)
        && match reduce_context_free_interpolations(value) {
            ContextFreeInterpolation::Static(value) => images::valid_static_reference(&value),
            ContextFreeInterpolation::Dynamic => true,
            ContextFreeInterpolation::Invalid => false,
        }
}

fn valid_container_value(value: &str) -> bool {
    valid_contextual_value(value, CONTAINER_CONTEXTS)
}

fn valid_contextual_value(value: &str, allowed_contexts: &[&str]) -> bool {
    !value.is_empty() && interpolated_expression_contexts_available(value, allowed_contexts)
}
