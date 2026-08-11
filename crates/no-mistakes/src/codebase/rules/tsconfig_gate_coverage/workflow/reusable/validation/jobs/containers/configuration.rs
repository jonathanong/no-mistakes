use serde_yaml::{Mapping, Value};

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
    ) && job
        .get("services")
        .and_then(Value::as_mapping)
        .is_none_or(|services| {
            services.values().all(|service| {
                service.as_mapping().is_some_and(|service| {
                    container_mapping_valid_for_inputs(
                        service,
                        ContainerKind::Service,
                        inputs,
                        environment,
                    )
                })
            })
        })
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
