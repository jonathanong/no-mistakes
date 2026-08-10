use super::super::super::super::expressions::interpolated_expression_contexts_available;
use super::fields::scalar_value_valid;
use serde_yaml::{Mapping, Value};
use std::collections::BTreeSet;

pub(crate) fn call_bindings_shape_valid(job: &Value) -> bool {
    job.as_mapping()
        .is_some_and(call_bindings_mapping_shape_valid)
}

pub(super) fn call_bindings_mapping_shape_valid(job: &Mapping) -> bool {
    binding_mapping_valid(job.get("with"))
        && match job.get("secrets") {
            Some(Value::String(value)) => value == "inherit",
            value => secret_binding_mapping_valid(value),
        }
}

fn binding_mapping_valid(value: Option<&Value>) -> bool {
    let Some(value) = value else {
        return true;
    };
    let Some(mapping) = value.as_mapping() else {
        return false;
    };
    unique_scalar_bindings(mapping, scalar_value_valid)
}

fn secret_binding_mapping_valid(value: Option<&Value>) -> bool {
    let Some(value) = value else {
        return true;
    };
    let Some(mapping) = value.as_mapping() else {
        return false;
    };
    unique_scalar_bindings(mapping, secret_value_valid)
}

fn unique_scalar_bindings(mapping: &Mapping, value_valid: fn(&Value) -> bool) -> bool {
    let mut names = BTreeSet::new();
    mapping.iter().all(|(name, value)| {
        name.as_str()
            .is_some_and(|name| names.insert(name.to_ascii_lowercase()))
            && value_valid(value)
    })
}

fn secret_value_valid(value: &Value) -> bool {
    matches!(value, Value::Bool(_) | Value::Number(_))
        || value.as_str().is_some_and(|value| {
            interpolated_expression_contexts_available(
                value,
                &[
                    "github", "needs", "strategy", "matrix", "secrets", "inputs", "vars",
                ],
            )
        })
}
