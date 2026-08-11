use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use serde_yaml::{Mapping, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(crate) fn catalog(
    root: &Path,
    tracked_paths: &[PathBuf],
    sources: &SourceStore,
) -> BTreeSet<String> {
    let mut descriptor_paths = BTreeMap::<String, (bool, &PathBuf)>::new();
    for path in tracked_paths {
        let name = path
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or_default();
        if !matches!(name, "action.yml" | "action.yaml") {
            continue;
        }
        let directory = relative_slash_path(
            root,
            path.parent()
                .expect("action metadata has a parent directory"),
        );
        let preferred = name == "action.yml";
        descriptor_paths
            .entry(directory)
            .and_modify(|(current_preferred, current)| {
                if preferred && !*current_preferred {
                    *current_preferred = true;
                    *current = path;
                }
            })
            .or_insert((preferred, path));
    }
    let descriptors = descriptor_paths
        .into_iter()
        .filter_map(|(directory, (_, path))| {
            let source = sources.read_path(path).ok()?;
            let metadata = serde_yaml::from_str(&source).ok()?;
            Some((directory, metadata))
        })
        .collect::<BTreeMap<_, _>>();
    let mut cache = BTreeMap::new();
    descriptors
        .keys()
        .filter(|directory| {
            action_directory_valid(directory, &descriptors, &mut BTreeSet::new(), &mut cache)
        })
        .cloned()
        .collect()
}

pub(super) fn workflow_targets_valid(workflow: &Value, catalog: &BTreeSet<String>) -> bool {
    workflow
        .get("jobs")
        .and_then(Value::as_mapping)
        .is_some_and(|jobs| {
            jobs.values().all(|job| {
                job.get("steps")
                    .and_then(Value::as_sequence)
                    .is_none_or(|steps| {
                        steps.iter().all(|step| {
                            step.get("uses")
                                .and_then(Value::as_str)
                                .and_then(|target| target.strip_prefix("./"))
                                .is_none_or(|target| catalog.contains(target))
                        })
                    })
            })
        })
}

fn action_directory_valid(
    directory: &str,
    descriptors: &BTreeMap<String, Value>,
    visiting: &mut BTreeSet<String>,
    cache: &mut BTreeMap<String, bool>,
) -> bool {
    if let Some(valid) = cache.get(directory) {
        return *valid;
    }
    if !visiting.insert(directory.to_string()) {
        return false;
    }
    let valid = descriptors
        .get(directory)
        .is_some_and(|metadata| valid_action_metadata(metadata, descriptors, visiting, cache));
    visiting.remove(directory);
    cache.insert(directory.to_string(), valid);
    valid
}

fn valid_action_metadata(
    metadata: &Value,
    descriptors: &BTreeMap<String, Value>,
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
                    && steps
                        .iter()
                        .all(|step| composite_step_valid(step, descriptors, visiting, cache))
            }),
        Some("docker") => nonempty_string(runs.get("image")),
        Some("node12" | "node16" | "node20" | "node24") => nonempty_string(runs.get("main")),
        _ => false,
    }
}

fn composite_step_valid(
    step: &Value,
    descriptors: &BTreeMap<String, Value>,
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
        return nonempty_string(Some(run)) && nonempty_string(step.get("shell"));
    }
    let target = step
        .get("uses")
        .and_then(Value::as_str)
        .expect("generic step validation requires a static action target");
    target
        .strip_prefix("./")
        .is_none_or(|directory| action_directory_valid(directory, descriptors, visiting, cache))
}

fn composite_steps_shape_valid(steps: &[Value]) -> bool {
    let mut job = Mapping::new();
    job.insert(
        Value::String("steps".to_string()),
        Value::Sequence(steps.to_vec()),
    );
    super::reusable::steps_shape_valid(&Value::Mapping(job))
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

#[cfg(test)]
mod tests;
