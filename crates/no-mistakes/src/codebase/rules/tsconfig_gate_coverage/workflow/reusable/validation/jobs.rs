use serde_yaml::{Mapping, Value};
use std::collections::BTreeSet;

pub(crate) fn steps_shape_valid(job: &Value) -> bool {
    let Some(steps) = job.get("steps") else {
        return true;
    };
    steps.as_sequence().is_some_and(|steps| {
        steps.iter().all(|step| {
            step.as_mapping().is_some_and(|step| {
                matches!(
                    (step.get("run"), step.get("uses")),
                    (Some(Value::String(command)), None)
                        | (None, Some(Value::String(command))) if !command.is_empty()
                )
            })
        })
    })
}

pub(crate) fn call_bindings_shape_valid(job: &Value) -> bool {
    binding_mapping_valid(job.get("with"))
        && match job.get("secrets") {
            Some(Value::String(value)) => value.eq_ignore_ascii_case("inherit"),
            value => binding_mapping_valid(value),
        }
}

fn binding_mapping_valid(value: Option<&Value>) -> bool {
    let Some(value) = value else {
        return true;
    };
    let Some(mapping) = value.as_mapping() else {
        return false;
    };
    unique_scalar_bindings(mapping)
}

fn unique_scalar_bindings(mapping: &Mapping) -> bool {
    let mut names = BTreeSet::new();
    mapping.iter().all(|(name, value)| {
        name.as_str()
            .is_some_and(|name| names.insert(name.to_ascii_lowercase()))
            && matches!(value, Value::Bool(_) | Value::Number(_) | Value::String(_))
    })
}

#[cfg(test)]
mod tests;
