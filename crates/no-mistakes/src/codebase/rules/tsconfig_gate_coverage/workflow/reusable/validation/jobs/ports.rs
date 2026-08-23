use serde_yaml::Value;
use std::net::IpAddr;

use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    resolve_static_interpolations, EnvironmentState, InputState,
};

use super::super::super::super::expressions::interpolated_expression_valid;

const MIN_PORT: u64 = 1;
const MAX_PORT: u64 = 65_535;
const DYNAMIC_EXPRESSION: &str = "\u{FDD0}";

pub(super) fn port_sequence_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value
            .as_sequence()
            .is_some_and(|ports| ports.iter().all(port_entry_valid))
    })
}

pub(super) fn port_sequence_valid_for_inputs(
    value: Option<&Value>,
    inputs: &InputState,
    environment: &EnvironmentState,
) -> bool {
    value.is_none_or(|value| {
        value.as_sequence().is_some_and(|ports| {
            ports.iter().all(|port| {
                port.as_str().map_or_else(
                    || port_entry_valid(port),
                    |port| {
                        resolve_static_interpolations(port, inputs, environment).map_or_else(
                            || port_mapping_valid(port),
                            |port| port_mapping_valid(&port),
                        )
                    },
                )
            })
        })
    })
}

fn port_entry_valid(value: &Value) -> bool {
    value.as_u64().is_some_and(valid_port_number) || value.as_str().is_some_and(port_mapping_valid)
}

fn port_mapping_valid(value: &str) -> bool {
    !value.is_empty()
        && !value.contains(DYNAMIC_EXPRESSION)
        && interpolated_expression_valid(value)
        && static_port_mapping_valid(&opaque_expression_form(value))
}

fn opaque_expression_form(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len());
    let mut remainder = value;
    while let Some(start) = remainder.find("${{") {
        normalized.push_str(&remainder[..start]);
        let expression_end = remainder[start + 3..]
            .find("}}")
            .expect("interpolated expression was validated")
            + start
            + 5;
        normalized.push_str(DYNAMIC_EXPRESSION);
        remainder = &remainder[expression_end..];
    }
    normalized.push_str(remainder);
    normalized
}

fn static_port_mapping_valid(value: &str) -> bool {
    let Some((mapping, protocol)) = split_protocol(value) else {
        return false;
    };
    if !protocol.is_none_or(protocol_valid) {
        return false;
    }
    if let Some(without_opening_bracket) = mapping.strip_prefix('[') {
        let Some((host, ports)) = without_opening_bracket.split_once("]:") else {
            return false;
        };
        return host_ip_valid(host) && mapping_pair_valid(ports);
    }

    let parts = mapping.split(':').collect::<Vec<_>>();
    match parts.as_slice() {
        [port] => port_spec_valid(port),
        [host, container] => port_mapping_pair_valid(host, container),
        [address, host, container] => {
            host_ip_valid(address) && port_mapping_pair_valid(host, container)
        }
        _ => false,
    }
}

fn split_protocol(value: &str) -> Option<(&str, Option<&str>)> {
    let mut parts = value.split('/');
    let mapping = parts.next()?;
    let protocol = parts.next();
    (parts.next().is_none()).then_some((mapping, protocol))
}

fn mapping_pair_valid(value: &str) -> bool {
    let mut parts = value.split(':');
    let Some(host) = parts.next() else {
        return false;
    };
    let Some(container) = parts.next() else {
        return false;
    };
    parts.next().is_none() && port_mapping_pair_valid(host, container)
}

fn port_mapping_pair_valid(host: &str, container: &str) -> bool {
    let Some(host_count) = port_spec_shape(host) else {
        return false;
    };
    let Some(container_count) = port_spec_shape(container) else {
        return false;
    };
    !matches!(
        (host_count, container_count),
        (PortSpec::Static(host_count), PortSpec::Static(container_count)) if host_count != container_count
    )
}

fn port_spec_valid(value: &str) -> bool {
    port_spec_shape(value).is_some()
}

fn port_spec_shape(value: &str) -> Option<PortSpec> {
    if complete_expression(value) {
        return Some(PortSpec::Dynamic);
    }
    if let Some((start, end)) = value.split_once('-') {
        let start = port_boundary(start)?;
        let end = port_boundary(end)?;
        return match (start, end) {
            (PortBoundary::Static(start), PortBoundary::Static(end)) => {
                (start <= end).then_some(PortSpec::Static(end - start + 1))
            }
            _ => Some(PortSpec::Dynamic),
        };
    }
    parse_port(value).map(|_| PortSpec::Static(1))
}

fn port_boundary(value: &str) -> Option<PortBoundary> {
    complete_expression(value)
        .then_some(PortBoundary::Dynamic)
        .or_else(|| parse_port(value).map(PortBoundary::Static))
}

fn parse_port(value: &str) -> Option<u64> {
    let port = value.parse::<u64>().ok()?;
    valid_port_number(port).then_some(port)
}

fn valid_port_number(port: u64) -> bool {
    (MIN_PORT..=MAX_PORT).contains(&port)
}

fn host_ip_valid(value: &str) -> bool {
    complete_expression(value) || value.parse::<IpAddr>().is_ok()
}

fn protocol_valid(value: &str) -> bool {
    complete_expression(value) || matches!(value, "tcp" | "udp" | "sctp")
}

fn complete_expression(value: &str) -> bool {
    value == DYNAMIC_EXPRESSION
}

#[derive(Clone, Copy)]
enum PortSpec {
    Static(u64),
    Dynamic,
}

#[derive(Clone, Copy)]
enum PortBoundary {
    Static(u64),
    Dynamic,
}

#[cfg(test)]
mod tests;
