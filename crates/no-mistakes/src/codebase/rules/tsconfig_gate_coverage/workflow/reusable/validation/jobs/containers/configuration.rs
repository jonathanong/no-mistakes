use serde_yaml::{Mapping, Value};
use std::collections::BTreeSet;

use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    resolve_static_interpolations, EnvironmentState, InputState,
};

use super::{images, options, ContainerKind};

pub(crate) fn container_configuration_valid_for_inputs(
    job: &Value,
    inputs: &InputState,
    environment: &EnvironmentState,
) -> bool {
    container_value_valid_for_inputs(
        job.get("container"),
        ContainerKind::Job,
        inputs,
        environment,
    ) && services_valid_for_inputs(job.get("services"), inputs, environment)
}

fn services_valid_for_inputs(
    value: Option<&Value>,
    inputs: &InputState,
    environment: &EnvironmentState,
) -> bool {
    let Some(services) = value else {
        return true;
    };
    let Some(services) = services.as_mapping() else {
        return false;
    };
    let mut bindings = BTreeSet::new();
    for service in services.values() {
        let Some(service) = service.as_mapping() else {
            return false;
        };
        if !service_mapping_valid_for_inputs(service, inputs, environment) {
            return false;
        }
        // Two services cannot publish the same statically known host binding.
        for binding in service_host_bindings(service, inputs, environment) {
            if !bindings.insert(binding) {
                return false;
            }
        }
    }
    true
}

fn service_mapping_valid_for_inputs(
    service: &Mapping,
    inputs: &InputState,
    environment: &EnvironmentState,
) -> bool {
    let Some(image) = service.get("image").and_then(Value::as_str) else {
        return false;
    };
    if !super::valid_container_image(image) {
        return false;
    }
    let Some(image) = resolve_static_interpolations(image, inputs, environment) else {
        return false;
    };
    // GitHub omits a service whose image resolves to the empty string. Its
    // remaining configuration therefore cannot make the job unrunnable.
    if image.is_empty() {
        return true;
    }
    images::valid_static_reference(&image)
        && container_mapping_valid_for_inputs(service, ContainerKind::Service, inputs, environment)
}

fn service_host_bindings(
    service: &Mapping,
    inputs: &InputState,
    environment: &EnvironmentState,
) -> Vec<String> {
    if service
        .get("image")
        .and_then(Value::as_str)
        .and_then(|image| resolve_static_interpolations(image, inputs, environment))
        .is_none_or(|image| image.is_empty())
    {
        return Vec::new();
    }
    service
        .get("ports")
        .and_then(Value::as_sequence)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter_map(|port| resolve_static_interpolations(port, inputs, environment))
        .filter_map(|port| static_host_binding(&port))
        .collect()
}

fn static_host_binding(port: &str) -> Option<String> {
    let (mapping, protocol) = port.split_once('/').map_or((port, "tcp"), |parts| parts);
    let mut parts = mapping.split(':');
    let host = parts.next()?;
    let container = parts.next()?;
    (parts.next().is_none() && host.parse::<u16>().is_ok() && container.parse::<u16>().is_ok())
        .then(|| format!("{host}/{protocol}"))
}

fn container_value_valid_for_inputs(
    value: Option<&Value>,
    kind: ContainerKind,
    inputs: &InputState,
    environment: &EnvironmentState,
) -> bool {
    value.is_none_or(|value| match value {
        Value::String(image) => container_image_valid_for_inputs(image, inputs, environment),
        Value::Mapping(container) => {
            container_mapping_valid_for_inputs(container, kind, inputs, environment)
        }
        _ => false,
    })
}

fn container_mapping_valid_for_inputs(
    container: &Mapping,
    kind: ContainerKind,
    inputs: &InputState,
    environment: &EnvironmentState,
) -> bool {
    container
        .get("image")
        .and_then(Value::as_str)
        .is_some_and(|image| container_image_valid_for_inputs(image, inputs, environment))
        && super::super::ports::port_sequence_valid_for_inputs(
            container.get("ports"),
            inputs,
            environment,
        )
        && super::volumes::valid_for_inputs(container.get("volumes"), inputs, environment)
        && credentials_valid_for_inputs(container.get("credentials"), inputs, environment)
        && options::valid_for_inputs(
            container.get("options").and_then(Value::as_str),
            kind,
            inputs,
            environment,
        )
}

fn container_image_valid_for_inputs(
    image: &str,
    inputs: &InputState,
    environment: &EnvironmentState,
) -> bool {
    super::valid_container_image(image)
        && resolve_static_interpolations(image, inputs, environment)
            .is_some_and(|image| images::valid_static_reference(&image))
}

fn credentials_valid_for_inputs(
    value: Option<&Value>,
    inputs: &InputState,
    environment: &EnvironmentState,
) -> bool {
    value.is_none_or(|value| {
        value.as_mapping().is_some_and(|credentials| {
            ["username", "password"].into_iter().all(|field| {
                credentials
                    .get(field)
                    .and_then(Value::as_str)
                    .is_some_and(|value| {
                        resolve_static_interpolations(value, inputs, environment)
                            .is_none_or(|value| !value.is_empty())
                    })
            })
        })
    })
}

#[cfg(test)]
mod tests;
