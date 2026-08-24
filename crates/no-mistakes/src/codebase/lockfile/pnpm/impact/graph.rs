use super::PnpmLocator;
use crate::codebase::lockfile::pnpm::{
    split_name_version, yaml_key_to_string, PnpmImporterDependency,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub(super) fn affected_locators(old: &str, new: &str) -> BTreeSet<PnpmLocator> {
    let old_graph = locator_graph(old);
    let new_graph = locator_graph(new);
    let mut affected = BTreeSet::new();
    for (graph, other) in [(&old_graph, &new_graph), (&new_graph, &old_graph)] {
        let changed = graph
            .records
            .keys()
            .filter(|key| graph.records.get(*key) != other.records.get(*key))
            .cloned()
            .collect();
        affected.extend(reverse_closure(&graph.reverse, changed));
    }
    affected
}

#[derive(Default)]
struct LocatorGraph {
    records: BTreeMap<PnpmLocator, Vec<serde_yaml::Value>>,
    reverse: BTreeMap<PnpmLocator, BTreeSet<PnpmLocator>>,
}

fn locator_graph(content: &str) -> LocatorGraph {
    let Ok(root) = serde_yaml::from_str::<serde_yaml::Value>(content) else {
        return LocatorGraph::default();
    };
    let mut graph = LocatorGraph::default();
    for section in ["packages", "snapshots"] {
        let Some(records) = root.get(section).and_then(serde_yaml::Value::as_mapping) else {
            continue;
        };
        for (key, value) in records {
            let Some(parent) = locator_from_key(&yaml_key_to_string(key)) else {
                continue;
            };
            graph
                .records
                .entry(parent.clone())
                .or_default()
                .push(value.clone());
            if section == "snapshots" {
                add_snapshot_base_package_edge(&mut graph, &parent);
            }
            add_dependency_edges(&mut graph, &parent, value);
        }
    }
    graph
}

/// pnpm v9 stores the resolved package record without peer context under
/// `packages`, then stores each peer-specific resolution under `snapshots`.
/// A base package change therefore affects every peer-context snapshot that
/// resolves that base package.
fn add_snapshot_base_package_edge(graph: &mut LocatorGraph, snapshot: &PnpmLocator) {
    if snapshot.peer_context.is_empty() {
        return;
    }
    graph
        .reverse
        .entry(PnpmLocator {
            name: snapshot.name.clone(),
            version: snapshot.version.clone(),
            peer_context: String::new(),
        })
        .or_default()
        .insert(snapshot.clone());
}

fn add_dependency_edges(graph: &mut LocatorGraph, parent: &PnpmLocator, value: &serde_yaml::Value) {
    for field in ["dependencies", "optionalDependencies", "peerDependencies"] {
        let Some(dependencies) = value.get(field).and_then(serde_yaml::Value::as_mapping) else {
            continue;
        };
        for (name, version) in dependencies {
            if let Some(child) = locator_from_dependency(&yaml_key_to_string(name), version) {
                graph
                    .reverse
                    .entry(child)
                    .or_default()
                    .insert(parent.clone());
            }
        }
    }
}

fn reverse_closure(
    reverse: &BTreeMap<PnpmLocator, BTreeSet<PnpmLocator>>,
    roots: Vec<PnpmLocator>,
) -> BTreeSet<PnpmLocator> {
    let mut found: BTreeSet<_> = roots.into_iter().collect();
    let mut queue: VecDeque<_> = found.iter().cloned().collect();
    while let Some(locator) = queue.pop_front() {
        if let Some(parents) = reverse.get(&locator) {
            for parent in parents {
                if found.insert(parent.clone()) {
                    queue.push_back(parent.clone());
                }
            }
        }
    }
    found
}

fn locator_from_key(key: &str) -> Option<PnpmLocator> {
    let (name, version) = split_name_version(key);
    let peer_context = peer_context(key, name, version);
    (!name.is_empty() && !version.is_empty()).then_some(PnpmLocator {
        name: name.to_string(),
        version: version.to_string(),
        peer_context,
    })
}

fn peer_context(key: &str, name: &str, version: &str) -> String {
    if let Some((_, context)) = key.split_once('(') {
        return context.trim_end_matches(')').to_string();
    }
    // pnpm v5/v6 encode peer contexts as `/name/version_peer@version`.
    key.strip_prefix('/')
        .unwrap_or(key)
        .strip_prefix(name)
        .and_then(|suffix| {
            suffix
                .strip_prefix('@')
                .or_else(|| suffix.strip_prefix('/'))
        })
        .and_then(|suffix| suffix.strip_prefix(version))
        .and_then(|suffix| suffix.strip_prefix('_'))
        .unwrap_or_default()
        .to_string()
}

fn locator_from_dependency(name: &str, value: &serde_yaml::Value) -> Option<PnpmLocator> {
    let version = value
        .as_str()
        .map(str::to_owned)
        .or_else(|| value.get("version").map(yaml_key_to_string))
        .or_else(|| {
            let scalar = yaml_key_to_string(value);
            (!scalar.is_empty()).then_some(scalar)
        })?;
    locator_from_key(&version).or_else(|| locator_from_key(&format!("{name}@{version}")))
}

pub(super) fn dependency_locator(dependency: &PnpmImporterDependency) -> Option<PnpmLocator> {
    let name = dependency
        .resolution_name
        .as_deref()
        .unwrap_or(&dependency.alias);
    locator_from_key(&dependency.version)
        .or_else(|| locator_from_key(&format!("{name}@{}", dependency.version)))
}
