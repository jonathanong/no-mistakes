use serde_yaml::{Mapping, Value};

use super::super::super::super::expressions::interpolated_expression_valid;
use super::fields::string_field_valid;
pub(super) fn scalar_mapping_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value.as_mapping().is_some_and(|mapping| {
            mapping
                .iter()
                .all(|(name, value)| name.is_string() && super::fields::scalar_value_valid(value))
        })
    })
}

pub(super) fn only_keys(mapping: &Mapping, allowed: &[&str]) -> bool {
    mapping
        .keys()
        .all(|key| key.as_str().is_some_and(|key| allowed.contains(&key)))
}

pub(super) fn runs_on_shape_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value
            .as_str()
            .is_some_and(valid_nonempty_interpolated_string)
            || value.as_sequence().is_some_and(|labels| {
                !labels.is_empty()
                    && labels.iter().all(|label| {
                        label
                            .as_str()
                            .is_some_and(valid_nonempty_interpolated_string)
                    })
            })
    })
}

pub(super) fn environment_shape_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value
            .as_str()
            .is_some_and(valid_nonempty_interpolated_string)
            || value.as_mapping().is_some_and(|environment| {
                only_keys(environment, &["name", "url"])
                    && environment
                        .get("name")
                        .and_then(Value::as_str)
                        .is_some_and(valid_nonempty_interpolated_string)
                    && environment.get("url").is_none_or(|url| {
                        url.as_str().is_some_and(valid_nonempty_interpolated_string)
                    })
            })
    })
}

pub(super) fn outputs_shape_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value.as_mapping().is_some_and(|outputs| {
            !outputs.is_empty()
                && outputs.iter().all(|(name, expression)| {
                    name.as_str().is_some_and(|name| !name.is_empty())
                        && expression
                            .as_str()
                            .is_some_and(valid_nonempty_interpolated_string)
                })
        })
    })
}

pub(super) fn container_shape_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value
            .as_str()
            .is_some_and(valid_nonempty_interpolated_string)
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
                    name.as_str().is_some_and(|name| !name.is_empty())
                        && (service
                            .as_str()
                            .is_some_and(valid_nonempty_interpolated_string)
                            || service
                                .as_mapping()
                                .is_some_and(container_mapping_shape_valid))
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
        .is_some_and(valid_nonempty_interpolated_string)
        && credentials_shape_valid(container.get("credentials"))
        && scalar_mapping_valid(container.get("env"))
        && scalar_sequence_valid(container.get("ports"))
        && string_sequence_valid(container.get("volumes"))
        && string_field_valid(container, "options")
}

fn credentials_shape_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value.as_mapping().is_some_and(|credentials| {
            only_keys(credentials, &["username", "password"])
                && credentials
                    .get("username")
                    .and_then(Value::as_str)
                    .is_some_and(valid_nonempty_interpolated_string)
                && credentials
                    .get("password")
                    .and_then(Value::as_str)
                    .is_some_and(valid_nonempty_interpolated_string)
        })
    })
}

fn scalar_sequence_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value
            .as_sequence()
            .is_some_and(|items| items.iter().all(super::fields::scalar_value_valid))
    })
}

fn string_sequence_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value.as_sequence().is_some_and(|items| {
            items.iter().all(|item| {
                item.as_str()
                    .is_some_and(valid_nonempty_interpolated_string)
            })
        })
    })
}

fn valid_nonempty_interpolated_string(value: &str) -> bool {
    !value.is_empty() && interpolated_expression_valid(value)
}
