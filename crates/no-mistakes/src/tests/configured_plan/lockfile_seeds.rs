// Traces lockfile package changes into test candidates for configured framework plans.
// This mirrors plan.rs §4b (non-framework path) but produces SelectedTest items
// that the caller can merge into the dependencies group.

use super::super::configured_plan_candidates::{bfs_path_find_set, merge_selected};
use super::super::plan::{impact_reason_label, path_confidence, relative_path, slash_node_name};
use super::super::{ImpactReason, SelectedTest, TestPlan, TestPlanGroupResult};
use super::fallback::{fallback_plan, FallbackRequest};
use anyhow::Result;
use no_mistakes::codebase::dependencies::graph::{DepGraph, NodeId};
use no_mistakes::codebase::test_discovery::DiscoveredTests;
use no_mistakes::codebase::workspaces::WorkspaceMap;
use std::collections::{BTreeMap, HashSet};
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

/// Merge lockfile-seeded candidates into `selected_map` and `group_results`, or return
/// a full-suite fallback plan when `global_config_fallback` is set and there are
/// genuinely untraceable tooling deps.
///
/// Returns `Ok(Some(plan))` if the caller should return that plan (fallback triggered),
/// or `Ok(None)` if candidates were merged successfully and the caller should continue.
#[allow(clippy::too_many_arguments)]
pub(super) fn apply_lockfile_seeds(
    root: &Path,
    seed_result: LockfileSeedResult,
    global_config_fallback: bool,
    all_tests: &[PathBuf],
    global_limit: usize,
    has_global_limit: bool,
    selected_map: &mut BTreeMap<PathBuf, SelectedTest>,
    used: &mut HashSet<String>,
    group_results: &mut Vec<TestPlanGroupResult>,
    discovered_tests: &DiscoveredTests,
) -> Result<Option<TestPlan>> {
    // Genuinely untraceable tooling deps (typescript, eslint, etc.) that have no
    // import-graph path to any test file — only fall back when the caller opted in.
    if !seed_result.untraceable_lockfiles.is_empty() && global_config_fallback {
        let lf = &seed_result.untraceable_lockfiles[0];
        let msg = format!(
            "`{}` changed a transitive dependency; falling back to full test suite",
            lf
        );
        let mut plan = fallback_plan(
            root,
            all_tests,
            FallbackRequest {
                group_type: "dependencies",
                via: "transitive dependency",
                changed_file: None,
                limit: global_limit,
                has_limit: has_global_limit,
                reason: msg,
            },
        );
        super::attach_targets(&mut plan, root, discovered_tests);
        return Ok(Some(plan));
    }
    // Merge traceable seeds into selected_map and the dependencies group result.
    for test in &seed_result.candidates {
        used.insert(test.test_file.clone());
        selected_map
            .entry(root.join(&test.test_file))
            .and_modify(|entry| merge_selected(entry, test))
            .or_insert_with(|| test.clone());
    }
    if !seed_result.candidates.is_empty() {
        // Append to existing dependencies group or push a new one. The group is at a
        // known position in default_groups, but be defensive with find+modify.
        let dep_names: Vec<String> = seed_result
            .candidates
            .iter()
            .map(|t| t.test_file.clone())
            .collect();
        if let Some(dep_group) = group_results
            .iter_mut()
            .find(|g| g.r#type == "dependencies")
        {
            for name in dep_names {
                if !dep_group.selected.contains(&name) {
                    dep_group.selected.push(name);
                }
            }
        } else {
            group_results.push(TestPlanGroupResult {
                r#type: "dependencies".to_string(),
                selected: dep_names,
                remaining: all_tests.len().saturating_sub(used.len()),
                limit: None,
            });
        }
    }
    Ok(None)
}
