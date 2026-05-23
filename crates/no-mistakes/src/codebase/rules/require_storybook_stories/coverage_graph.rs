use super::types::component_key;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::ts_resolver::{normalize_path, ImportResolver};
use crate::codebase::ts_source::relative_slash_path;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

pub(super) fn transitive_covered_components(
    root: &Path,
    shared: &CheckFactMap,
    direct: &BTreeSet<String>,
    component_keys: &HashSet<String>,
) -> BTreeSet<String> {
    let mut covered = direct.clone();
    let mut queue: VecDeque<String> = direct.iter().cloned().collect();
    let component_children = component_children(root, shared);
    let components_by_file = components_by_file(component_keys);
    while let Some(key) = queue.pop_front() {
        enqueue_same_file_components(&key, &components_by_file, &mut covered, &mut queue);
        let Some(children) = component_children.get(&key) else {
            continue;
        };
        for child in children {
            if component_keys.contains(child) && covered.insert(child.clone()) {
                queue.push_back(child.clone());
            }
        }
    }
    covered
}

fn enqueue_same_file_components(
    key: &str,
    components_by_file: &HashMap<String, Vec<String>>,
    covered: &mut BTreeSet<String>,
    queue: &mut VecDeque<String>,
) {
    let Some((file, _)) = key.split_once('#') else {
        return;
    };
    let Some(siblings) = components_by_file.get(file) else {
        return;
    };
    for sibling in siblings {
        if covered.insert(sibling.clone()) {
            queue.push_back(sibling.clone());
        }
    }
}

fn components_by_file(component_keys: &HashSet<String>) -> HashMap<String, Vec<String>> {
    let mut out: HashMap<String, Vec<String>> = HashMap::new();
    for key in component_keys {
        if let Some((file, _)) = key.split_once('#') {
            out.entry(file.to_string()).or_default().push(key.clone());
        }
    }
    out
}

fn component_children(root: &Path, shared: &CheckFactMap) -> HashMap<String, Vec<String>> {
    let mut out = HashMap::new();
    for facts in shared.ts.values() {
        let Some(react) = facts.react.as_ref() else {
            continue;
        };
        for component in &react.components {
            let key = component_key(&component.file, &component.name);
            let children = component
                .children
                .iter()
                .map(|child| {
                    let file = relative_slash_path(root, &normalize_path(&root.join(&child.file)));
                    component_key(&file, &child.name)
                })
                .collect();
            out.insert(key, children);
        }
    }
    out
}

pub(super) fn dynamic_or_mock_boundary_files(
    shared: &CheckFactMap,
    resolver: &ImportResolver<'_>,
) -> HashSet<PathBuf> {
    let mut out = HashSet::new();
    for (file, facts) in &shared.ts {
        let Some(dynamic) = facts.dynamic_imports.as_ref() else {
            continue;
        };
        for specifier in dynamic
            .dynamic_imports
            .iter()
            .filter_map(|import| import.specifier.as_deref())
            .chain(dynamic.mock_specifiers.iter().map(String::as_str))
        {
            if let Some(resolved) = resolver.resolve(specifier, file) {
                out.insert(normalize_path(&resolved));
            }
        }
    }
    out
}
