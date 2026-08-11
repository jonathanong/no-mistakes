use super::*;
use crate::codebase::rules::tsconfig_gate_coverage::command_scan;
use serde_yaml::Mapping;

const MAX_COMPOSITE_ACTION_DEPTH: usize = 10;

pub(super) fn action_directory_valid(
    directory: &str,
    descriptors: &BTreeMap<String, Value>,
    tracked: &BTreeSet<String>,
    visiting: &mut BTreeSet<String>,
    cache: &mut BTreeMap<String, bool>,
) -> bool {
    let root = visiting.is_empty();
    if root {
        if let Some(valid) = cache.get(directory) {
            return *valid;
        }
    }
    if visiting.len() >= MAX_COMPOSITE_ACTION_DEPTH {
        return false;
    }
    if !visiting.insert(directory.to_string()) {
        return false;
    }
    let valid = descriptors.get(directory).is_some_and(|metadata| {
        valid_action_metadata(metadata, directory, descriptors, tracked, visiting, cache)
    });
    visiting.remove(directory);
    if root {
        cache.insert(directory.to_string(), valid);
    }
    valid
}

fn valid_action_metadata(
    metadata: &Value,
    directory: &str,
    descriptors: &BTreeMap<String, Value>,
    tracked: &BTreeSet<String>,
    visiting: &mut BTreeSet<String>,
    cache: &mut BTreeMap<String, bool>,
) -> bool {
    let Some(metadata) = metadata.as_mapping() else {
        return false;
    };
    if !nonempty_string(metadata.get("name")) || !nonempty_string(metadata.get("description")) {
        return false;
    }
    let Some(runs) = metadata.get("runs").and_then(Value::as_mapping) else {
        return false;
    };
    match runs.get("using").and_then(Value::as_str) {
        Some("composite") => runs
            .get("steps")
            .and_then(Value::as_sequence)
            .is_some_and(|steps| {
                !steps.is_empty()
                    && composite_steps_shape_valid(steps)
                    && steps.iter().all(|step| {
                        composite_step_valid(step, descriptors, tracked, visiting, cache)
                    })
            }),
        Some("docker") => nonempty_string(runs.get("image")),
        Some("node12" | "node16" | "node20" | "node24") => runs
            .get("main")
            .and_then(Value::as_str)
            .and_then(|main| action_file(directory, main))
            .is_some_and(|main| tracked.contains(&main)),
        _ => false,
    }
}

fn action_file(directory: &str, path: &str) -> Option<String> {
    let path = command_scan::normalize_repo_relative(path)?;
    command_scan::normalize_repo_relative(&format!("{directory}/{path}"))
}

fn composite_step_valid(
    step: &Value,
    descriptors: &BTreeMap<String, Value>,
    tracked: &BTreeSet<String>,
    visiting: &mut BTreeSet<String>,
    cache: &mut BTreeMap<String, bool>,
) -> bool {
    const KEYS: &[&str] = &[
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
    let step = step
        .as_mapping()
        .expect("generic step validation requires a mapping");
    if !only_keys(step, KEYS) {
        return false;
    }
    if let Some(run) = step.get("run") {
        return nonempty_string(Some(run))
            && nonempty_string(step.get("shell"))
            && (!composite_step_may_run(step)
                || !run
                    .as_str()
                    .is_some_and(command_scan::shell_body_has_static_failure));
    }
    let target = step
        .get("uses")
        .and_then(Value::as_str)
        .expect("generic step validation requires a static action target");
    target.strip_prefix("./").is_none_or(|directory| {
        action_directory_valid(directory, descriptors, tracked, visiting, cache)
    })
}

fn composite_step_may_run(step: &Mapping) -> bool {
    match step.get("if") {
        Some(Value::Bool(false)) => false,
        Some(Value::String(expression)) => {
            super::super::conditions::expression_bool(expression, &BTreeMap::new())
                != super::super::conditions::StaticBool::False
        }
        None | Some(_) => true,
    }
}

fn composite_steps_shape_valid(steps: &[Value]) -> bool {
    let mut job = Mapping::new();
    job.insert(
        Value::String("steps".to_string()),
        Value::Sequence(steps.to_vec()),
    );
    super::super::reusable::steps_shape_valid(&Value::Mapping(job))
}

fn only_keys(mapping: &Mapping, keys: &[&str]) -> bool {
    mapping
        .keys()
        .all(|key| key.as_str().is_some_and(|key| keys.contains(&key)))
}

fn nonempty_string(value: Option<&Value>) -> bool {
    value
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty())
}
