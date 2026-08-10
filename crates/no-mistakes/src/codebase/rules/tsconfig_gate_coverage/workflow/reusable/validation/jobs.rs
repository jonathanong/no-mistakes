use serde_yaml::{Mapping, Value};
use std::collections::BTreeSet;

pub(crate) fn steps_shape_valid(job: &Value) -> bool {
    let Some(steps) = job.get("steps") else {
        return job.get("uses").is_some();
    };
    steps.as_sequence().is_some_and(|steps| {
        !steps.is_empty()
            && steps.iter().all(|step| {
                step.as_mapping().is_some_and(|step| {
                    matches!(
                        (step.get("run"), step.get("uses")),
                        (Some(Value::String(command)), None) if !command.is_empty()
                    ) || matches!(
                        (step.get("run"), step.get("uses")),
                        (None, Some(Value::String(target))) if action_target_valid(target)
                    )
                })
            })
    })
}

fn action_target_valid(target: &str) -> bool {
    if target.contains("${{") || target.chars().any(char::is_whitespace) {
        return false;
    }
    if let Some(path) = target.strip_prefix("./") {
        return !path.is_empty()
            && !path.contains('\\')
            && path
                .split('/')
                .all(|segment| !matches!(segment, "" | "." | ".."));
    }
    if let Some(image) = target.strip_prefix("docker://") {
        return !image.is_empty();
    }
    let Some((path, reference)) = target.rsplit_once('@') else {
        return false;
    };
    let mut segments = path.split('/');
    segments.next().is_some_and(|owner| !owner.is_empty())
        && segments
            .next()
            .is_some_and(|repository| !repository.is_empty())
        && segments.all(|segment| !segment.is_empty())
        && !reference.is_empty()
}

pub(crate) fn call_bindings_shape_valid(job: &Value) -> bool {
    binding_mapping_valid(job.get("with"))
        && match job.get("secrets") {
            Some(Value::String(value)) => value == "inherit",
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
