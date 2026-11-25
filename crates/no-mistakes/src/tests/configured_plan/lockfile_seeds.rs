// Traces lockfile package changes into test candidates for configured framework plans.
// This mirrors plan.rs §4b (non-framework path) but produces SelectedTest items
// that the caller can merge into the dependencies group.

use super::super::configured_plan_candidates::{bfs_path_find_set, merge_selected};
use super::super::plan::{impact_reason_label, path_confidence, relative_path, slash_node_name};
use super::super::{
    via_details_from_edges, ImpactReason, SelectedTest, TestPlan, TestPlanGroupResult,
};
use super::fallback::{fallback_plan, FallbackRequest};
use anyhow::Result;
use no_mistakes::codebase::dependencies::graph::{DepGraph, NodeId};
use no_mistakes::codebase::test_discovery::DiscoveredTests;
use no_mistakes::codebase::workspaces::WorkspaceMap;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;

pub(super) struct LockfileSeedResult {
    pub(super) candidates: Vec<SelectedTest>,
    /// Changed dependencies that had no import-graph path to any test
    /// (e.g. tooling deps like `typescript`, `eslint`).
    pub(super) untraceable_dependencies: Vec<UntraceableLockfileDependency>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct UntraceableLockfileDependency {
    pub(super) package_name: String,
    pub(super) lockfile: String,
}

pub(crate) fn first_untraceable_lockfile_dependency(
    prepared: &crate::tests::prepared_plan::PreparedTestPlanRequest,
    discovered_tests: &[PathBuf],
) -> Option<String> {
    if prepared.lockfile_changed_packages.is_empty() {
        return None;
    }
    let Ok(graph) = prepared.graph() else {
        return Some("Failed to build dependency graph for lockfile check".to_string());
    };
    let all_test_set: HashSet<PathBuf> = discovered_tests.iter().cloned().collect();
    let result = lockfile_seed_candidates(
        &prepared.root,
        &prepared.lockfile_changed_packages,
        &prepared.workspace_map,
        graph,
        &all_test_set,
        &HashSet::new(),
    );
    result.untraceable_dependencies.into_iter().next().map(|dep| {
        format!(
            "`{}` changed a dependency without a causal path to a configured test; full-suite selection requires global fallback opt-in",
            dep.lockfile
        )
    })
}

pub(super) fn lockfile_seed_candidates(
    root: &Path,
    lockfile_changed_packages: &[(String, String, Vec<PathBuf>)],
    workspace_map: &WorkspaceMap,
    graph: &DepGraph,
    all_test_set: &HashSet<PathBuf>,
    used: &HashSet<String>,
) -> LockfileSeedResult {
    let mut candidates_map: std::collections::BTreeMap<String, SelectedTest> =
        std::collections::BTreeMap::new();
    let mut untraceable_dependencies = std::collections::BTreeSet::new();

    for (pkg_name, lockfile_rel, manifest_scope) in lockfile_changed_packages {
        // External modules use Module(name); workspace packages use their entry file.
        let start_node = {
            let module_node = NodeId::module(pkg_name.clone());
            if graph.has_reverse_node(&module_node) {
                module_node
            } else if let Some(entry) = workspace_map.resolve_package(pkg_name) {
                NodeId::file(entry.clone())
            } else {
                untraceable_dependencies.insert(UntraceableLockfileDependency {
                    package_name: pkg_name.clone(),
                    lockfile: lockfile_rel.clone(),
                });
                continue;
            }
        };

        let start_nodes = package_scope::scoped_importer_start_nodes(
            graph,
            &start_node,
            manifest_scope,
            workspace_map,
        );
        let mut seeded_any_test = false;
        for start_node in start_nodes {
            if let Some(test_path) = start_node
                .node
                .as_file()
                .filter(|path| all_test_set.contains(*path))
            {
                seeded_any_test = true;
                let rel_test = relative_path(root, test_path);
                let (path, edges) =
                    package_scope::prefix_package_path(root, &start_node, Vec::new(), Vec::new());
                insert_lockfile_candidate(&mut candidates_map, rel_test, lockfile_rel, path, edges);
            }
            let (reachable_tests, path_parents) =
                bfs_path_find_set(graph, &start_node.node, all_test_set);
            for (test_node, edge_path) in reachable_tests {
                let NodeId::File(test_path) = &test_node else {
                    continue;
                };
                seeded_any_test = true;
                let rel_test = relative_path(root, test_path);
                if used.contains(&rel_test) {
                    continue;
                }
                let mut node_chain = Vec::new();
                let mut curr = test_node.clone();
                node_chain.push(slash_node_name(&curr, root));
                while let Some((parent, _)) = path_parents.get(&curr) {
                    node_chain.push(slash_node_name(parent, root));
                    curr = parent.clone();
                }
                node_chain.reverse();
                let (path, edges) =
                    package_scope::prefix_package_path(root, &start_node, node_chain, edge_path);
                insert_lockfile_candidate(&mut candidates_map, rel_test, lockfile_rel, path, edges);
            }
        }
        if !seeded_any_test {
            untraceable_dependencies.insert(UntraceableLockfileDependency {
                package_name: pkg_name.clone(),
                lockfile: lockfile_rel.clone(),
            });
        }
    }

    LockfileSeedResult {
        candidates: candidates_map.into_values().collect(),
        untraceable_dependencies: untraceable_dependencies.into_iter().collect(),
    }
}

fn insert_lockfile_candidate(
    candidates: &mut BTreeMap<String, SelectedTest>,
    test_file: String,
    lockfile_rel: &str,
    path: Vec<String>,
    edges: Vec<no_mistakes::codebase::dependencies::graph::EdgeKind>,
) {
    let confidence = path_confidence(&edges);
    let reason = ImpactReason {
        changed_file: lockfile_rel.to_string(),
        path,
        via: edges
            .iter()
            .map(|kind| impact_reason_label(*kind).to_string())
            .collect(),
        via_details: via_details_from_edges(&edges),
    };
    let entry = candidates
        .entry(test_file.clone())
        .or_insert_with(|| SelectedTest {
            test_file,
            confidence,
            targets: Vec::new(),
            reasons: Vec::new(),
        });
    if confidence > entry.confidence {
        entry.confidence = confidence;
    }
    if !entry.reasons.contains(&reason) {
        entry.reasons.push(reason);
    }
}

#[path = "lockfile_seeds/dotnet.rs"]
mod dotnet;
pub(super) use dotnet::dotnet_dependency_seed_candidates;

#[path = "lockfile_seeds/swift.rs"]
mod swift;
pub(super) use swift::{swift_manifest_seed_candidates, swift_resolved_seed_candidates};

#[path = "lockfile_seeds/merge.rs"]
mod merge;
pub(super) use merge::merge_lockfile_seed_candidates;

#[path = "lockfile_seeds/apply.rs"]
mod apply;
pub(super) use apply::apply_lockfile_seeds;

#[path = "lockfile_seeds/package_scope.rs"]
mod package_scope;
