//! Local composite actions referenced by `uses: ./...`.
//!
//! Secret inventory must not parse every visible `action.yml`. Vendored and
//! fixture copies are included only when a workflow or an already referenced
//! action points at them.

use super::ParsedWorkflowSet;
use crate::codebase::ts_source::SourceStore;
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(crate) fn referenced_local_action_paths(
    root: &Path,
    workflows: &ParsedWorkflowSet,
    candidates: impl IntoIterator<Item = PathBuf>,
    sources: &SourceStore,
) -> Vec<PathBuf> {
    let descriptors = action_descriptors(root, candidates);
    let mut pending = Vec::new();
    for document in &workflows.documents {
        if let Ok(value) = &document.value {
            collect_local_uses(value, &mut pending);
        }
    }
    let mut selected = BTreeSet::new();
    let mut seen_uses = BTreeSet::new();
    while let Some(uses) = pending.pop() {
        if !seen_uses.insert(uses.clone()) {
            continue;
        }
        let Some(relative) = local_use_relative(&uses) else {
            continue;
        };
        for descriptor in descriptor_relatives(&relative) {
            let Some(absolute) = descriptors.get(&descriptor) else {
                continue;
            };
            if !selected.insert(absolute.clone()) {
                continue;
            }
            if let Ok(value) = sources.parse_yaml_path(absolute) {
                collect_local_uses(value.as_ref(), &mut pending);
            }
        }
    }
    selected.into_iter().collect()
}

fn action_descriptors(
    root: &Path,
    candidates: impl IntoIterator<Item = PathBuf>,
) -> BTreeMap<String, PathBuf> {
    let mut descriptors = BTreeMap::new();
    for path in candidates {
        let Some(name) = path.file_name() else {
            continue;
        };
        // Non-UTF8 names cannot be `action.yml`. Fold them into the name check.
        let name = name.to_string_lossy();
        if name != "action.yml" && name != "action.yaml" {
            continue;
        }
        let relative = crate::codebase::ci_graph::relative_slash(root, &path);
        descriptors.insert(relative, path);
    }
    descriptors
}

fn collect_local_uses(value: &Value, uses: &mut Vec<String>) {
    match value {
        Value::Mapping(mapping) => {
            for (key, value) in mapping {
                if key.as_str() == Some("uses") {
                    if let Some(action) = value.as_str() {
                        let action = action.trim();
                        if action.starts_with("./") {
                            uses.push(action.to_owned());
                        }
                    }
                }
                collect_local_uses(value, uses);
            }
        }
        Value::Sequence(values) => {
            for value in values {
                collect_local_uses(value, uses);
            }
        }
        Value::Tagged(tagged) => collect_local_uses(&tagged.value, uses),
        _ => {}
    }
}

fn local_use_relative(uses: &str) -> Option<String> {
    let uses = uses.trim().replace('\\', "/");
    let rest = uses.strip_prefix("./")?.trim_matches('/');
    if rest.is_empty() || rest.split('/').any(|part| part == "..") {
        return None;
    }
    Some(rest.to_owned())
}

fn descriptor_relatives(path: &str) -> Vec<String> {
    let lower = path.to_ascii_lowercase();
    if lower == "action.yml"
        || lower == "action.yaml"
        || lower.ends_with("/action.yml")
        || lower.ends_with("/action.yaml")
    {
        return vec![path.to_owned()];
    }
    if lower.ends_with(".yml") || lower.ends_with(".yaml") {
        return Vec::new();
    }
    vec![format!("{path}/action.yml"), format!("{path}/action.yaml")]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_use_paths_reject_non_local_empty_and_parent_segments() {
        assert_eq!(local_use_relative("actions/checkout@v4"), None);
        assert_eq!(local_use_relative("./"), None);
        assert_eq!(local_use_relative("./../secret/action.yml"), None);
        assert_eq!(
            local_use_relative(r".\actions\outer"),
            Some("actions/outer".to_owned())
        );
    }

    #[test]
    fn descriptor_index_ignores_paths_without_file_names() {
        assert!(action_descriptors(Path::new("/repo"), [PathBuf::from("/")]).is_empty());
    }

    #[test]
    fn tagged_and_non_string_uses_do_not_invent_local_actions() {
        use serde_yaml::value::{Tag, TaggedValue};

        let mut mapping = serde_yaml::Mapping::new();
        mapping.insert(
            Value::String("uses".into()),
            Value::String("  ./actions/outer  ".into()),
        );
        let tagged = Value::Tagged(Box::new(TaggedValue {
            tag: Tag::new("!relative"),
            value: Value::Mapping(mapping),
        }));
        let mut uses = Vec::new();
        collect_local_uses(&tagged, &mut uses);
        assert_eq!(uses, vec!["./actions/outer".to_owned()]);

        let mut ignored = serde_yaml::Mapping::new();
        ignored.insert(Value::String("uses".into()), Value::Sequence(Vec::new()));
        uses.clear();
        collect_local_uses(&Value::Mapping(ignored), &mut uses);
        assert!(uses.is_empty());
    }

    #[test]
    fn descriptors_follow_directories_and_ignore_workflow_files() {
        assert_eq!(
            descriptor_relatives("actions/outer"),
            ["actions/outer/action.yml", "actions/outer/action.yaml"]
        );
        assert_eq!(
            descriptor_relatives("actions/outer/action.yml"),
            ["actions/outer/action.yml"]
        );
        assert_eq!(descriptor_relatives("action.yaml"), ["action.yaml"]);
        assert!(descriptor_relatives(".github/workflows/reusable.yml").is_empty());
        assert!(descriptor_relatives(".github/workflows/reusable.yaml").is_empty());
    }
}
