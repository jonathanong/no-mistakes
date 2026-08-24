use super::super::model::WorkflowTopology;
use super::yaml::yaml_at;
use git2::{Repository, Tree};
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn action_job_users(
    repo: &Repository,
    base: &Tree<'_>,
    head: &Tree<'_>,
    changed: &BTreeSet<String>,
    base_topology: &WorkflowTopology,
    head_topology: &WorkflowTopology,
) -> BTreeSet<String> {
    let mut impacted = changed.clone();
    for tree in [base, head] {
        let descriptors = action_descriptors(repo, tree);
        let mut expanded = true;
        while expanded {
            expanded = false;
            for action in &descriptors {
                if !impacted.contains(action)
                    && action_uses(repo, tree, action)
                        .iter()
                        .any(|used| impacted.contains(used))
                {
                    expanded = impacted.insert(action.clone()) || expanded;
                }
            }
        }
    }
    base_topology
        .jobs
        .iter()
        .chain(&head_topology.jobs)
        .filter(|job| {
            job.steps.iter().any(|step| {
                step.uses.as_deref().is_some_and(|uses| {
                    uses.starts_with("./")
                        && impacted.contains(uses.trim_start_matches("./").trim_end_matches('/'))
                })
            })
        })
        .map(|job| job.id.clone())
        .collect()
}

fn action_descriptors(repo: &Repository, tree: &Tree<'_>) -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    collect_action_descriptors(repo, tree, "", &mut paths);
    paths
}

fn collect_action_descriptors(
    repo: &Repository,
    tree: &Tree<'_>,
    prefix: &str,
    paths: &mut BTreeSet<String>,
) {
    for entry in tree {
        let name = entry.name().unwrap_or_default();
        let path = if prefix.is_empty() {
            name.to_owned()
        } else {
            format!("{prefix}/{name}")
        };
        if entry.kind() == Some(git2::ObjectType::Tree) {
            if path == ".github" || path.starts_with(".github/actions") {
                collect_action_descriptors(
                    repo,
                    &entry
                        .to_object(repo)
                        .ok()
                        .and_then(|object| object.peel_to_tree().ok())
                        .expect("tree entry"),
                    &path,
                    paths,
                );
            }
        } else if path.starts_with(".github/actions/")
            && (path.ends_with("/action.yml") || path.ends_with("/action.yaml"))
        {
            paths.insert(
                path.rsplit_once('/')
                    .expect("action descriptor parent")
                    .0
                    .to_owned(),
            );
        }
    }
}

fn action_uses(repo: &Repository, tree: &Tree<'_>, action: &str) -> BTreeSet<String> {
    [
        format!("{action}/action.yml"),
        format!("{action}/action.yaml"),
    ]
    .iter()
    .filter_map(|path| yaml_at(repo, tree, path))
    .flat_map(|value| local_uses(&value))
    .collect()
}

fn local_uses(value: &serde_yaml::Value) -> BTreeSet<String> {
    let mut uses = BTreeSet::new();
    collect_uses(value, &mut uses);
    uses
}

fn collect_uses(value: &serde_yaml::Value, uses: &mut BTreeSet<String>) {
    match value {
        serde_yaml::Value::Mapping(map) => {
            for (key, value) in map {
                if key.as_str() == Some("uses") {
                    if let Some(action) = value.as_str().filter(|action| action.starts_with("./")) {
                        uses.insert(
                            action
                                .trim_start_matches("./")
                                .trim_end_matches('/')
                                .to_owned(),
                        );
                    }
                }
                collect_uses(value, uses);
            }
        }
        serde_yaml::Value::Sequence(values) => {
            for value in values {
                collect_uses(value, uses);
            }
        }
        _ => {}
    }
}

pub(super) fn action_descriptors_for_path(
    base: &Tree<'_>,
    head: &Tree<'_>,
    path: &str,
) -> BTreeSet<String> {
    let mut result = BTreeSet::new();
    let mut cursor = Path::new(path).parent().map(Path::to_path_buf);
    while let Some(directory_path) = cursor {
        let next = directory_path.parent().map(Path::to_path_buf);
        let directory = directory_path.to_string_lossy().replace('\\', "/");
        if !directory.starts_with(".github/actions/") {
            break;
        }
        if [base, head].iter().any(|tree| {
            ["action.yml", "action.yaml"].iter().any(|name| {
                tree.get_path(Path::new(&format!("{directory}/{name}")))
                    .is_ok()
            })
        }) {
            result.insert(directory);
            break;
        }
        cursor = next;
    }
    result
}
