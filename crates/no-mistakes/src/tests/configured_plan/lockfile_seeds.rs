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

#[cfg(test)]
mod tests;

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
                via_details: via_details_from_edges(&edge_path),
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

pub(super) fn merge_lockfile_seed_candidates(
    root: &Path,
    seeds: &[SelectedTest],
    candidates: &mut Vec<SelectedTest>,
    used: &HashSet<String>,
    selected: &mut BTreeMap<PathBuf, SelectedTest>,
) {
    for seed in seeds {
        if used.contains(&seed.test_file) {
            if let Some(existing) = selected.get_mut(&root.join(&seed.test_file)) {
                merge_selected(existing, seed);
                existing.targets.clear();
            }
            continue;
        }
        if let Some(existing) = candidates
            .iter_mut()
            .find(|candidate| candidate.test_file == seed.test_file)
        {
            // Lockfile tracing independently selected this test, so retain
            // every owning runner target while merging its reason.
            merge_selected(existing, seed);
            existing.targets.clear();
        } else {
            candidates.push(seed.clone());
        }
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
    // Compute how many seeds can be added, respecting global and group limits.
    let mut max_to_add = if has_global_limit {
        global_limit.saturating_sub(used.len())
    } else {
        usize::MAX
    };
    if let Some(dep_group) = group_results.iter().find(|g| g.r#type == "dependencies") {
        if let Some(limit_val) = dep_group.limit {
            let remaining_group = limit_val.saturating_sub(dep_group.selected.len());
            max_to_add = max_to_add.min(remaining_group);
        }
    }
    // Merge traceable seeds into selected_map and the dependencies group result.
    let mut added: Vec<String> = Vec::new();
    for test in &seed_result.candidates {
        if used.contains(&test.test_file) {
            if let Some(existing) = selected_map.get_mut(&root.join(&test.test_file)) {
                merge_selected(existing, test);
                // Lockfile tracing selected this test independently, so its
                // execution is no longer restricted to a targeted subset.
                existing.targets.clear();
            }
            continue;
        }
        if added.len() >= max_to_add {
            // Keep scanning: later seeds may already be selected and still
            // need their independent lockfile reasons merged at zero budget.
            continue;
        }
        if used.insert(test.test_file.clone()) {
            selected_map
                .entry(root.join(&test.test_file))
                .and_modify(|entry| merge_selected(entry, test))
                .or_insert_with(|| test.clone());
            added.push(test.test_file.clone());
        }
    }
    if !added.is_empty() {
        // Append to existing dependencies group or push a new one. The group is at a
        // known position in default_groups, but be defensive with find+modify.
        if let Some(dep_group) = group_results
            .iter_mut()
            .find(|g| g.r#type == "dependencies")
        {
            for name in added {
                if !dep_group.selected.contains(&name) {
                    dep_group.selected.push(name);
                }
            }
            dep_group.remaining = all_tests.len().saturating_sub(used.len());
        } else {
            group_results.push(TestPlanGroupResult {
                r#type: "dependencies".to_string(),
                selected: added,
                remaining: all_tests.len().saturating_sub(used.len()),
                limit: None,
            });
        }
    }
    Ok(None)
}
