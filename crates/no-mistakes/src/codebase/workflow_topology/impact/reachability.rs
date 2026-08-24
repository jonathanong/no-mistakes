use super::yaml::yaml_at;
use git2::{Repository, Tree};
use std::collections::BTreeSet;

pub(super) fn reachable_actions(
    repo: &Repository,
    base: &Tree<'_>,
    head: &Tree<'_>,
    entry: &str,
) -> BTreeSet<String> {
    [base, head]
        .into_iter()
        .flat_map(|tree| reachable_actions_in_tree(repo, tree, entry))
        .collect()
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
        for descriptor in [format!("{path}/action.yml"), format!("{path}/action.yaml")] {
            if let Some(action) = yaml_at(repo, tree, &descriptor) {
                collect_reachable_uses(&action, tree, repo, workflows, actions);
            }
        }
    }
}

fn is_reusable_workflow(path: &str) -> bool {
    path.starts_with(".github/workflows/") && (path.ends_with(".yml") || path.ends_with(".yaml"))
}
