use super::yaml::yaml_at;
use git2::{Repository, Tree};
use std::collections::BTreeSet;

pub(super) fn reachable_actions(
    repo: &Repository,
    base: &Tree<'_>,
    head: &Tree<'_>,
    entry: &str,
) -> BTreeSet<String> {
    let mut visited = BTreeSet::new();
    for tree in [base, head] {
        if let Some(value) = yaml_at(repo, tree, entry) {
            collect_reachable_uses(&value, tree, repo, &mut visited);
        }
    }
    visited
}

fn collect_reachable_uses(
    value: &serde_yaml::Value,
    tree: &Tree<'_>,
    repo: &Repository,
    visited: &mut BTreeSet<String>,
) {
    match value {
        serde_yaml::Value::Mapping(map) => {
            for (key, value) in map {
                if key.as_str() == Some("uses") {
                    if let Some(uses) = value.as_str().filter(|uses| uses.starts_with("./")) {
                        let action = uses
                            .trim_start_matches("./")
                            .trim_end_matches('/')
                            .to_string();
                        if visited.insert(action.clone()) {
                            for descriptor in [
                                format!("{action}/action.yml"),
                                format!("{action}/action.yaml"),
                            ] {
                                if let Some(action_yaml) = yaml_at(repo, tree, &descriptor) {
                                    collect_reachable_uses(&action_yaml, tree, repo, visited);
                                }
                            }
                        }
                    }
                }
                collect_reachable_uses(value, tree, repo, visited);
            }
        }
        serde_yaml::Value::Sequence(values) => {
            for value in values {
                collect_reachable_uses(value, tree, repo, visited);
            }
        }
        _ => {}
    }
}
