use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};

mod composite_shape;
mod execution;
mod icons;
mod shape;

use execution::{composite_step_valid, docker_action_image_valid};
use shape::{
    action_inputs_valid, branding_valid, nonempty_string, only_keys, outputs_valid,
    runs_shape_valid,
};

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
    if !only_keys(
        metadata,
        &[
            "name",
            "author",
            "description",
            "inputs",
            "outputs",
            "runs",
            "branding",
        ],
    ) || !nonempty_string(metadata.get("name"))
        || !nonempty_string(metadata.get("description"))
        || !metadata
            .get("author")
            .is_none_or(|value| nonempty_string(Some(value)))
        || !action_inputs_valid(metadata.get("inputs"))
        || !branding_valid(metadata.get("branding"))
    {
        return false;
    }
    let Some(runs) = metadata.get("runs").and_then(Value::as_mapping) else {
        return false;
    };
    match runs.get("using").and_then(Value::as_str) {
        Some("composite") => {
            runs_shape_valid(runs, "composite")
                && outputs_valid(metadata.get("outputs"), true)
                && runs
                    .get("steps")
                    .and_then(Value::as_sequence)
                    .is_some_and(|steps| {
                        !steps.is_empty()
                            && execution::composite_steps_shape_valid(steps)
                            && steps.iter().all(|step| {
                                composite_step_valid(step, descriptors, tracked, visiting, cache)
                            })
                    })
        }
        Some("docker") => {
            runs_shape_valid(runs, "docker")
                && outputs_valid(metadata.get("outputs"), false)
                && docker_action_image_valid(runs, directory, tracked)
                && runs.get("pre-entrypoint").is_none_or(|entrypoint| {
                    entrypoint
                        .as_str()
                        .and_then(|entrypoint| execution::action_file(directory, entrypoint))
                        .is_some_and(|entrypoint| tracked.contains(&entrypoint))
                })
        }
        Some("node20" | "node24") => {
            runs_shape_valid(runs, "node")
                && outputs_valid(metadata.get("outputs"), false)
                // `pre` is rejected by the node runs key allowlist, so only `post` is checked here.
                && runs.get("post").is_none_or(|post| {
                    post.as_str()
                        .and_then(|post| execution::action_file(directory, post))
                        .is_some_and(|post| tracked.contains(&post))
                })
                && runs
                    .get("main")
                    .and_then(Value::as_str)
                    .and_then(|main| execution::action_file(directory, main))
                    .is_some_and(|main| tracked.contains(&main))
        }
        _ => false,
    }
}
