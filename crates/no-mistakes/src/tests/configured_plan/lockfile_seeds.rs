// Traces lockfile package changes into test candidates for configured framework plans.
// This mirrors plan.rs §4b (non-framework path) but produces SelectedTest items
// that the caller can merge into the dependencies group.

use super::super::configured_plan_candidates::bfs_path_find_set;
use super::super::plan::{impact_reason_label, path_confidence, relative_path, slash_node_name};
use super::super::{ImpactReason, SelectedTest};
use no_mistakes::codebase::dependencies::graph::{DepGraph, NodeId};
use no_mistakes::codebase::workspaces::WorkspaceMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) struct LockfileSeedResult {
    pub(super) candidates: Vec<SelectedTest>,
    /// Relative lockfile paths that had no import-graph path to any test
    /// (e.g. tooling deps like `typescript`, `eslint`).
    pub(super) untraceable_lockfiles: Vec<String>,
}

pub(super) fn lockfile_seed_candidates(
    root: &Path,
    lockfile_changed_packages: &[(String, String)], // (pkg_name, lockfile_rel)
    workspace_map: &WorkspaceMap,
    graph: &DepGraph,
    all_test_set: &HashSet<PathBuf>,
    used: &HashSet<String>,
) -> LockfileSeedResult {
    let mut candidates_map: std::collections::BTreeMap<String, SelectedTest> =
        std::collections::BTreeMap::new();
    let mut untraceable_lockfiles: Vec<String> = Vec::new();

    for (pkg_name, lockfile_rel) in lockfile_changed_packages {
        // Try Module(name) first (external packages create these nodes from import edges).
        // Fall back to File(workspace_entry) for workspace packages whose graph edges
        // point at the entry file rather than a Module node.
        let start_node = {
            let module_node = NodeId::Module(pkg_name.clone());
            if graph.has_reverse_node(&module_node) {
                module_node
            } else if let Some(entry) = workspace_map.resolve_package(pkg_name) {
                NodeId::File(entry.clone())
            } else {
                // Package referenced in lockfile but absent from graph and workspace map.
                // Could be a transitive dep or a tooling dep with no import-graph path.
                if !untraceable_lockfiles.contains(lockfile_rel) {
                    untraceable_lockfiles.push(lockfile_rel.clone());
                }
                continue;
            }
        };

        let (reachable_tests, path_parents) = bfs_path_find_set(graph, &start_node, all_test_set);

        // Package is present in graph but no test file is reachable — likely a tooling dep
        // (typescript, eslint, jest) whose version bump affects how tests run but has no
        // import-graph path to any test file.
        if reachable_tests.is_empty() {
            if !untraceable_lockfiles.contains(lockfile_rel) {
                untraceable_lockfiles.push(lockfile_rel.clone());
            }
            continue;
        }

        for (test_node, edge_path) in reachable_tests {
            let NodeId::File(test_path) = &test_node else {
                continue;
            };
            let rel_test = relative_path(root, test_path);
            if used.contains(&rel_test) {
                continue;
            }
            let path_conf = path_confidence(&edge_path);

            let mut node_chain = Vec::new();
            let mut curr = test_node.clone();
            node_chain.push(slash_node_name(&curr, root));
            while let Some((parent, _)) = path_parents.get(&curr) {
                node_chain.push(slash_node_name(parent, root));
                curr = parent.clone();
            }
            node_chain.reverse();

            let via_strings: Vec<String> = edge_path
                .iter()
                .map(|k| impact_reason_label(*k).to_string())
                .collect();

            let reason = ImpactReason {
                changed_file: lockfile_rel.clone(),
                path: node_chain,
                via: via_strings,
            };

            let entry = candidates_map
                .entry(rel_test.clone())
                .or_insert_with(|| SelectedTest {
                    test_file: rel_test.clone(),
                    confidence: path_conf,
                    targets: Vec::new(),
                    reasons: Vec::new(),
                });
            if path_conf > entry.confidence {
                entry.confidence = path_conf;
            }
            if !entry.reasons.contains(&reason) {
                entry.reasons.push(reason);
            }
        }
    }

    LockfileSeedResult {
        candidates: candidates_map.into_values().collect(),
        untraceable_lockfiles,
    }
}
