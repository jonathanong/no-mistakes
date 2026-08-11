use super::shape::{nonempty_string, only_keys};
use crate::codebase::rules::tsconfig_gate_coverage::command_scan;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::{
    reduce_context_free_interpolations, ContextFreeInterpolation,
};
use serde_yaml::{Mapping, Value};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn docker_action_image_valid(
    runs: &Mapping,
    directory: &str,
    tracked: &BTreeSet<String>,
) -> bool {
    let Some(image) = runs.get("image").and_then(Value::as_str) else {
        return false;
    };
    if image.is_empty() || image.trim() != image || image.eq_ignore_ascii_case("docker://") {
        return false;
    }
    if !dockerfile_image(image) {
        return true;
    }
    action_file(directory, image).is_some_and(|dockerfile| tracked.contains(&dockerfile))
}

fn dockerfile_image(image: &str) -> bool {
    if image
        .get(.."docker://".len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("docker://"))
    {
        return false;
    }
    let name = image.rsplit('/').next().unwrap_or(image);
    name.get(.."Dockerfile.".len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("Dockerfile."))
        || name
            .get(name.len().saturating_sub("Dockerfile".len())..)
            .is_some_and(|suffix| suffix.eq_ignore_ascii_case("Dockerfile"))
}

pub(super) fn action_file(directory: &str, path: &str) -> Option<String> {
    let path = command_scan::normalize_repo_relative(path)?;
    if directory.is_empty() {
        Some(path)
    } else {
        command_scan::normalize_repo_relative(&format!("{directory}/{path}"))
    }
}

pub(super) fn composite_step_valid(
    step: &Value,
    descriptors: &BTreeMap<String, Value>,
    tracked: &BTreeSet<String>,
    visiting: &mut BTreeSet<String>,
    cache: &mut BTreeMap<String, bool>,
) -> bool {
    const KEYS: &[&str] = &[
        "continue-on-error",
        "env",
        "id",
        "if",
        "name",
        "run",
        "shell",
        "uses",
        "with",
        "working-directory",
    ];
    let Some(step) = step.as_mapping() else {
        return false;
    };
    if !only_keys(step, KEYS) {
        return false;
    }
    if let Some(run) = step.get("run") {
        return nonempty_string(Some(run))
            && nonempty_string(step.get("shell"))
            && (composite_step_continues_on_error(step)
                || !composite_step_may_run(step)
                || (composite_step_working_directory_valid(step, tracked)
                    && !run
                        .as_str()
                        .is_some_and(command_scan::shell_body_has_static_failure)));
    }
    let Some(target) = step.get("uses").and_then(Value::as_str) else {
        return false;
    };
    target.strip_prefix("./").is_none_or(|directory| {
        super::super::action_directory_valid(directory, descriptors, tracked, visiting, cache)
    })
}

fn composite_step_working_directory_valid(step: &Mapping, tracked: &BTreeSet<String>) -> bool {
    let Some(value) = step.get("working-directory") else {
        return true;
    };
    let Some(value) = value.as_str() else {
        return false;
    };
    match reduce_context_free_interpolations(value) {
        ContextFreeInterpolation::Dynamic => true,
        ContextFreeInterpolation::Invalid => false,
        ContextFreeInterpolation::Static(path) => {
            let Some(path) = command_scan::normalize_repo_relative(&path) else {
                return false;
            };
            path == "."
                || tracked
                    .iter()
                    .any(|tracked| tracked.starts_with(&format!("{path}/")))
        }
    }
}

fn composite_step_continues_on_error(step: &Mapping) -> bool {
    match step.get("continue-on-error") {
        Some(Value::Bool(value)) => *value,
        Some(Value::String(expression)) => {
            super::super::super::conditions::expression_bool(expression, &BTreeMap::new())
                == super::super::super::conditions::StaticBool::True
        }
        _ => false,
    }
}

fn composite_step_may_run(step: &Mapping) -> bool {
    match step.get("if") {
        Some(Value::Bool(false)) => false,
        Some(Value::String(expression)) => {
            super::super::super::conditions::expression_bool(expression, &BTreeMap::new())
                != super::super::super::conditions::StaticBool::False
        }
        None | Some(_) => true,
    }
}

pub(super) fn composite_steps_shape_valid(steps: &[Value]) -> bool {
    let mut job = Mapping::new();
    job.insert(
        Value::String("steps".to_string()),
        Value::Sequence(steps.to_vec()),
    );
    super::super::super::reusable::steps_shape_valid(&Value::Mapping(job))
}
