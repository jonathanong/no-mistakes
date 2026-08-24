use super::actions::action_descriptor_paths;
use super::yaml::yaml_at;
use git2::{Repository, Tree};
use std::collections::BTreeSet;
use std::path::Path;

pub(super) struct ReachableActions {
    pub(super) paths: BTreeSet<String>,
    pub(super) unresolved: BTreeSet<String>,
}

pub(super) fn reachable_actions(
    repo: &Repository,
    base: &Tree<'_>,
    head: &Tree<'_>,
    entry: &str,
) -> ReachableActions {
    let paths = [base, head]
        .into_iter()
        .flat_map(|tree| reachable_actions_in_tree(repo, tree, entry))
        .collect::<BTreeSet<_>>();
    let unresolved = paths
        .iter()
        .filter(|action| {
            let statuses = [base, head].map(|tree| action_descriptor_status(repo, tree, action));
            statuses.contains(&ActionDescriptorStatus::Invalid)
                || !statuses.contains(&ActionDescriptorStatus::Valid)
        })
        .cloned()
        .collect();
    ReachableActions { paths, unresolved }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActionDescriptorStatus {
    Absent,
    Valid,
    Invalid,
}

fn action_descriptor_status(
    repo: &Repository,
    tree: &Tree<'_>,
    action: &str,
) -> ActionDescriptorStatus {
    let mut found = false;
    for descriptor in action_descriptor_paths(action) {
        let Ok(entry) = tree.get_path(Path::new(&descriptor)) else {
            continue;
        };
        found = true;
        let Ok(object) = entry.to_object(repo) else {
            return ActionDescriptorStatus::Invalid;
        };
        let Ok(blob) = object.peel_to_blob() else {
            return ActionDescriptorStatus::Invalid;
        };
        let Ok(value) = serde_yaml::from_slice::<serde_yaml::Value>(blob.content()) else {
            return ActionDescriptorStatus::Invalid;
        };
        let Some(mapping) = value.as_mapping() else {
            return ActionDescriptorStatus::Invalid;
        };
        if !mapping
            .iter()
            .any(|(key, value)| key.as_str() == Some("runs") && value.is_mapping())
        {
            return ActionDescriptorStatus::Invalid;
        }
    }
    if found {
        ActionDescriptorStatus::Valid
    } else {
        ActionDescriptorStatus::Absent
    }
}

fn reachable_actions_in_tree(repo: &Repository, tree: &Tree<'_>, entry: &str) -> BTreeSet<String> {
    let mut actions = BTreeSet::new();
    let mut workflows = BTreeSet::from([entry.to_owned()]);
    if let Some(value) = yaml_at(repo, tree, entry) {
        collect_reachable_uses(&value, tree, repo, &mut workflows, &mut actions);
    }
    actions
}

fn collect_reachable_uses(
    value: &serde_yaml::Value,
    tree: &Tree<'_>,
    repo: &Repository,
    workflows: &mut BTreeSet<String>,
    actions: &mut BTreeSet<String>,
) {
    match value {
        serde_yaml::Value::Mapping(map) => {
            for (key, value) in map {
                if key.as_str() == Some("uses") {
                    collect_local_use(value, tree, repo, workflows, actions);
                }
                collect_reachable_uses(value, tree, repo, workflows, actions);
            }
        }
        serde_yaml::Value::Sequence(values) => {
            for value in values {
                collect_reachable_uses(value, tree, repo, workflows, actions);
            }
        }
        _ => {}
    }
}

fn collect_local_use(
    value: &serde_yaml::Value,
    tree: &Tree<'_>,
    repo: &Repository,
    workflows: &mut BTreeSet<String>,
    actions: &mut BTreeSet<String>,
) {
    let Some(path) = value
        .as_str()
        .filter(|uses| uses.starts_with("./"))
        .map(|uses| uses.trim_start_matches("./").trim_end_matches('/'))
    else {
        return;
    };
    if is_reusable_workflow(path) {
        if workflows.insert(path.to_owned()) {
            if let Some(workflow) = yaml_at(repo, tree, path) {
                collect_reachable_uses(&workflow, tree, repo, workflows, actions);
            }
        }
    } else if actions.insert(path.to_owned()) {
        for descriptor in action_descriptor_paths(path) {
            if let Some(action) = yaml_at(repo, tree, &descriptor) {
                collect_reachable_uses(&action, tree, repo, workflows, actions);
            }
        }
    }
}

fn is_reusable_workflow(path: &str) -> bool {
    path.starts_with(".github/workflows/") && (path.ends_with(".yml") || path.ends_with(".yaml"))
}
