use super::{
    bfs_path_find, global_config_fallback, impact_reason_label, path_confidence, relative_path,
    slash_node_name, via_details_from_edges, warnings, Confidence, ImpactReason, PlanArgs,
    SelectedTest, TestPlan, Warning, WarningKey,
};
use crate::tests::configured_plan::native_semantic_seeds::NativeSemanticSeedResult;
use crate::tests::prepared_plan::PreparedTestPlanRequest;
use crate::tests::warning_key;
use no_mistakes::codebase::dependencies::graph::EdgeKind;
use no_mistakes::codebase::dependencies::graph::{DepGraph, NodeId};
use no_mistakes::codebase::test_filter::TestFileFilter;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub(super) struct DependencySeedContext<'a> {
    pub(super) args: &'a PlanArgs,
    pub(super) prepared: &'a PreparedTestPlanRequest,
    pub(super) graph: &'a DepGraph,
    pub(super) test_filter: &'a TestFileFilter,
    pub(super) all_test_files: &'a [PathBuf],
    pub(super) native_semantic_seeds: &'a NativeSemanticSeedResult,
}

pub(super) struct DependencySeedState<'a> {
    pub(super) selected_map: &'a mut HashMap<PathBuf, SelectedTest>,
    pub(super) warnings: &'a mut Vec<Warning>,
    pub(super) warnings_seen: &'a mut HashSet<WarningKey>,
}

pub(super) fn trace_and_fallback(
    context: DependencySeedContext<'_>,
    state: DependencySeedState<'_>,
) -> Option<TestPlan> {
    let DependencySeedContext {
        args,
        prepared,
        graph,
        test_filter,
        all_test_files,
        native_semantic_seeds,
    } = context;
    let DependencySeedState {
        selected_map,
        warnings,
        warnings_seen,
    } = state;
    let root = &prepared.root;
    let workspace_map = &prepared.workspace_map;
    let mut untraceable_lockfile_files = Vec::new();
    for (pkg_name, lockfile_rel, manifest_scope) in &prepared.lockfile_changed_packages {
        let Some(start_node) = lockfile_start_node(
            graph,
            workspace_map.resolve_package(pkg_name),
            pkg_name,
            lockfile_rel,
            &mut untraceable_lockfile_files,
        ) else {
            continue;
        };
        let start_nodes = scoped_start_nodes(graph, workspace_map, manifest_scope, start_node);
        let mut seeded_any_test = false;
        for scoped_start in start_nodes {
            trace_lockfile_seed(
                LockfileSeedContext {
                    graph,
                    test_filter,
                    root,
                    lockfile_rel,
                },
                LockfileSeedState {
                    selected_map,
                    seeded_any_test: &mut seeded_any_test,
                },
                &scoped_start.node,
                scoped_start.prefix.as_ref(),
            );
        }
        if !seeded_any_test && !untraceable_lockfile_files.contains(lockfile_rel) {
            untraceable_lockfile_files.push(lockfile_rel.clone());
        }
    }

    for file in &untraceable_lockfile_files {
        let warning = Warning {
            r#type: "package-dependency-untraceable".to_string(),
            message: format!(
                "`{file}` changed a dependency without a causal path to a configured test; full-suite selection requires global fallback opt-in"
            ),
            file: file.clone(),
            line: None,
        };
        if warnings_seen.insert(warning_key(&warning)) {
            warnings.push(warning);
        }
    }
    if global_config_fallback(args) && !untraceable_lockfile_files.is_empty() {
        warnings::extend_analysis_warnings(prepared, warnings, warnings_seen);
        let file = &untraceable_lockfile_files[0];
        return Some(fallback_plan(
            all_test_files,
            root,
            warnings,
            file,
            "transitive dependency",
            format!("`{file}` changed a transitive dependency; falling back to full test suite"),
        ));
    }

    native_semantic_seeds.extend_warnings(warnings, warnings_seen);
    if global_config_fallback(args) {
        if let Some(file) = native_semantic_seeds.first_untraceable() {
            warnings::extend_analysis_warnings(prepared, warnings, warnings_seen);
            return Some(fallback_plan(
                all_test_files,
                root,
                warnings,
                file,
                "native dependency",
                format!(
                    "`{file}` changed a native dependency without a causal test path; falling back to full test suite"
                ),
            ));
        }
    }
    None
}

fn lockfile_start_node(
    graph: &DepGraph,
    workspace_entry: Option<&PathBuf>,
    package_name: &str,
    lockfile_rel: &str,
    untraceable: &mut Vec<String>,
) -> Option<NodeId> {
    let module_node = NodeId::module(package_name.to_string());
    if graph.has_reverse_node(&module_node) {
        return Some(module_node);
    }
    if let Some(entry) = workspace_entry {
        return Some(NodeId::file(entry.clone()));
    }
    if !untraceable.contains(&lockfile_rel.to_string()) {
        untraceable.push(lockfile_rel.to_string());
    }
    None
}

struct ScopedStart {
    node: NodeId,
    /// The original package-to-importer edge, retained after applying manifest scope.
    prefix: Option<(NodeId, EdgeKind)>,
}

fn scoped_start_nodes(
    graph: &DepGraph,
    workspace_map: &no_mistakes::codebase::workspaces::WorkspaceMap,
    manifest_scope: &[PathBuf],
    start_node: NodeId,
) -> Vec<ScopedStart> {
    if manifest_scope.is_empty() {
        return vec![ScopedStart {
            node: start_node,
            prefix: None,
        }];
    }
    graph
        .dependents_of_node(&start_node)
        .into_iter()
        .flatten()
        .filter_map(|(node, kind)| {
            node.as_file()
                .filter(|path| path_matches_manifest_scope(workspace_map, manifest_scope, path))
                .map(|_| ScopedStart {
                    node: node.clone(),
                    prefix: Some((start_node.clone(), *kind)),
                })
        })
        .collect()
}

fn path_matches_manifest_scope(
    workspace_map: &no_mistakes::codebase::workspaces::WorkspaceMap,
    manifest_scope: &[PathBuf],
    path: &std::path::Path,
) -> bool {
    let nearest_workspace = workspace_map
        .packages
        .iter()
        .filter(|package| path.starts_with(&package.dir))
        .max_by_key(|package| package.dir.components().count());
    manifest_scope.iter().any(|manifest| {
        let owner = manifest
            .parent()
            .expect("package manifest has a parent directory");
        nearest_workspace.map_or_else(|| path.starts_with(owner), |package| package.dir == owner)
    })
}

struct LockfileSeedContext<'a> {
    graph: &'a DepGraph,
    test_filter: &'a TestFileFilter,
    root: &'a std::path::Path,
    lockfile_rel: &'a str,
}

struct LockfileSeedState<'a> {
    selected_map: &'a mut HashMap<PathBuf, SelectedTest>,
    seeded_any_test: &'a mut bool,
}

fn trace_lockfile_seed(
    context: LockfileSeedContext<'_>,
    state: LockfileSeedState<'_>,
    start: &NodeId,
    prefix: Option<&(NodeId, EdgeKind)>,
) {
    let LockfileSeedContext {
        graph,
        test_filter,
        root,
        lockfile_rel,
    } = context;
    let LockfileSeedState {
        selected_map,
        seeded_any_test,
    } = state;
    if let Some(test_path) = start
        .as_file()
        .filter(|path| test_filter.is_match(root, path))
    {
        *seeded_any_test = true;
        let (path, edges) = prefixed_path(root, start, prefix, Vec::new(), Vec::new());
        insert_lockfile_candidate(root, test_path, lockfile_rel, path, edges, selected_map);
    }
    let (reachable_tests, path_parents) = bfs_path_find(graph, start, test_filter, root);
    for (test_node, edge_path) in reachable_tests {
        let test_path = match &test_node {
            NodeId::File(path) => path.to_path_buf(),
            _ => continue,
        };
        *seeded_any_test = true;
        let mut node_chain = vec![slash_node_name(&test_node, root)];
        let mut current = test_node.clone();
        while let Some((parent, _)) = path_parents.get(&current) {
            node_chain.push(slash_node_name(parent, root));
            current = parent.clone();
        }
        node_chain.reverse();
        let (path, edges) = prefixed_path(root, start, prefix, node_chain, edge_path);
        insert_lockfile_candidate(root, &test_path, lockfile_rel, path, edges, selected_map);
    }
}

fn prefixed_path(
    root: &std::path::Path,
    start: &NodeId,
    prefix: Option<&(NodeId, EdgeKind)>,
    mut path: Vec<String>,
    mut edges: Vec<EdgeKind>,
) -> (Vec<String>, Vec<EdgeKind>) {
    if path.is_empty() {
        path.push(slash_node_name(start, root));
    }
    if let Some((original_start, kind)) = prefix {
        path.insert(0, slash_node_name(original_start, root));
        edges.insert(0, *kind);
    }
    (path, edges)
}

fn insert_lockfile_candidate(
    root: &std::path::Path,
    test_path: &std::path::Path,
    lockfile_rel: &str,
    path: Vec<String>,
    edges: Vec<EdgeKind>,
    selected_map: &mut HashMap<PathBuf, SelectedTest>,
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
    let entry = selected_map
        .entry(test_path.to_path_buf())
        .or_insert_with(|| SelectedTest {
            test_file: relative_path(root, test_path),
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

fn fallback_plan(
    all_test_files: &[PathBuf],
    root: &std::path::Path,
    warnings: &[Warning],
    file: &str,
    via: &str,
    fallback_reason: String,
) -> TestPlan {
    let mut selected_tests: Vec<SelectedTest> = all_test_files
        .iter()
        .map(|test| {
            let test_file = relative_path(root, test);
            SelectedTest {
                test_file: test_file.clone(),
                confidence: Confidence::High,
                targets: Vec::new(),
                reasons: vec![ImpactReason {
                    changed_file: file.to_string(),
                    path: vec![file.to_string(), test_file],
                    via: vec![via.to_string()],
                    via_details: Vec::new(),
                }],
            }
        })
        .collect();
    selected_tests.sort_by(|left, right| left.test_file.cmp(&right.test_file));
    TestPlan {
        changed_files: Vec::new(),
        selected_tests,
        groups: Vec::new(),
        warnings: warnings.to_vec(),
        fallback_triggered: true,
        fallback_reason: Some(fallback_reason),
        ..Default::default()
    }
}
