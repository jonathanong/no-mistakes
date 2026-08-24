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

pub(super) fn dotnet_dependency_seed_candidates(
    root: &Path,
    artifacts: &[crate::tests::dotnet_dependency_changes::DotnetDependencyArtifact],
    facts: Option<&no_mistakes::codebase::dotnet::DotnetFactMap>,
    visible_paths: &[PathBuf],
    graph: &DepGraph,
    all_test_set: &HashSet<PathBuf>,
) -> Vec<SelectedTest> {
    let mut result = Vec::new();
    for artifact in artifacts {
        let seeds = match artifact.kind {
            crate::tests::dotnet_dependency_changes::DotnetArtifactKind::CentralPackages => facts
                .map(|facts| {
                    central_package_seed_roots(
                        root,
                        &artifact.path,
                        &artifact.changed_dependencies,
                        facts,
                        visible_paths,
                    )
                })
                .unwrap_or_default(),
            crate::tests::dotnet_dependency_changes::DotnetArtifactKind::Project => {
                artifact.owning_project.clone().into_iter().collect()
            }
            crate::tests::dotnet_dependency_changes::DotnetArtifactKind::Lockfile => facts
                .map(|facts| lockfile_seed_roots(artifact, facts))
                .unwrap_or_default(),
        };
        for seed in seeds {
            let (tests, parents) =
                bfs_path_find_set(graph, &NodeId::file(seed.clone()), all_test_set);
            for (node, edges) in tests {
                let NodeId::File(test) = node else { continue };
                let mut path = vec![slash_node_name(&NodeId::file(test.clone()), root)];
                let mut current = NodeId::file(test.clone());
                while let Some((parent, _)) = parents.get(&current) {
                    path.push(slash_node_name(parent, root));
                    current = parent.clone()
                }
                path.reverse();
                result.push(SelectedTest {
                    test_file: relative_path(root, &test),
                    confidence: path_confidence(&edges),
                    targets: Vec::new(),
                    reasons: vec![ImpactReason {
                        changed_file: relative_path(root, &artifact.path),
                        path,
                        via: edges
                            .iter()
                            .map(|edge| impact_reason_label(*edge).to_string())
                            .collect(),
                        via_details: via_details_from_edges(&edges),
                    }],
                });
            }
        }
    }
    result
}

fn lockfile_seed_roots(
    artifact: &crate::tests::dotnet_dependency_changes::DotnetDependencyArtifact,
    facts: &no_mistakes::codebase::dotnet::DotnetFactMap,
) -> Vec<PathBuf> {
    let Some(owner) = artifact.owning_project.as_ref() else {
        return Vec::new();
    };
    facts
        .projects
        .values()
        .filter(|project| project.project_dir == *owner)
        .map(|project| project.project_path.clone())
        .collect()
}

#[cfg(test)]
mod dotnet_semantic_seed_tests {
    use super::*;
    use no_mistakes::codebase::dotnet::{DotnetFactMap, DotnetProjectFacts};
    use std::collections::BTreeSet;

    #[test]
    fn central_consumers_are_scoped_to_nearest_props() {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/dotnet-semantic-seeds/fixture");
        let fixture = crate::test_support::materialize_saved_fixture(&source);
        let root = fixture.path().canonicalize().unwrap();
        let mut facts = DotnetFactMap::default();
        for (name, package) in [("core", "Core.Only"), ("app", "App.Only")] {
            let dir = root.join(name);
            let path = dir.join(format!("{name}.csproj"));
            facts.projects.insert(
                path.clone(),
                DotnetProjectFacts {
                    project_path: path,
                    project_dir: dir,
                    package_references: BTreeSet::from([package.to_string()]),
                    ..Default::default()
                },
            );
        }
        let central_files = [
            root.join("Directory.Packages.props"),
            root.join("app/Directory.Packages.props"),
        ];
        assert_eq!(
            central_package_seed_roots(
                &root,
                &root.join("app/Directory.Packages.props"),
                &BTreeSet::from(["App.Only".to_string()]),
                &facts,
                &central_files,
            ),
            vec![root.join("app/app.csproj")]
        );
        assert_eq!(
            central_package_seed_roots(
                &root,
                &root.join("Directory.Packages.props"),
                &BTreeSet::from(["Core.Only".to_string()]),
                &facts,
                &central_files,
            ),
            vec![root.join("core/core.csproj")]
        );
        assert!(central_package_seed_roots(
            &root,
            &root.join("Directory.Packages.props"),
            &BTreeSet::from(["App.Only".to_string()]),
            &facts,
            &central_files,
        )
        .is_empty());
        assert_eq!(
            central_package_seed_roots(
                &root,
                &root.join("app/Directory.Packages.props"),
                &BTreeSet::from(["app.only".to_string()]),
                &facts,
                &central_files,
            ),
            vec![root.join("app/app.csproj")],
            "NuGet package identities are case-insensitive"
        );
        assert!(central_package_seed_roots(
            &root,
            &root.join("Directory.Packages.props"),
            &BTreeSet::from(["Other".to_string()]),
            &facts,
            &central_files,
        )
        .is_empty());
    }

    #[test]
    fn lockfile_seeds_every_project_that_shares_its_owner_directory() {
        let owner = PathBuf::from("/repo/apps");
        let mut facts = DotnetFactMap::default();
        for name in ["First", "Second"] {
            let project_path = owner.join(format!("{name}.csproj"));
            facts.projects.insert(
                project_path.clone(),
                DotnetProjectFacts {
                    project_path,
                    project_dir: owner.clone(),
                    ..Default::default()
                },
            );
        }
        let artifact = crate::tests::dotnet_dependency_changes::DotnetDependencyArtifact {
            path: owner.join("packages.lock.json"),
            kind: crate::tests::dotnet_dependency_changes::DotnetArtifactKind::Lockfile,
            changed_dependencies: BTreeSet::new(),
            owning_project: Some(owner),
        };

        assert_eq!(
            lockfile_seed_roots(&artifact, &facts),
            [
                PathBuf::from("/repo/apps/First.csproj"),
                PathBuf::from("/repo/apps/Second.csproj"),
            ]
        );
    }
}

pub(super) fn central_package_seed_roots(
    root: &Path,
    props: &Path,
    packages: &std::collections::BTreeSet<String>,
    facts: &no_mistakes::codebase::dotnet::DotnetFactMap,
    visible_paths: &[PathBuf],
) -> Vec<PathBuf> {
    facts
        .projects
        .values()
        .filter(|project| {
            project.package_references.iter().any(|reference| {
                packages
                    .iter()
                    .any(|package| reference.eq_ignore_ascii_case(package))
            }) && nearest_central_props(root, &project.project_dir, visible_paths).as_deref()
                == Some(props)
        })
        .map(|project| project.project_path.clone())
        .collect()
}
fn nearest_central_props(root: &Path, dir: &Path, visible_paths: &[PathBuf]) -> Option<PathBuf> {
    let mut current = dir.to_path_buf();
    loop {
        let candidate = current.join("Directory.Packages.props");
        if visible_paths.contains(&candidate) {
            return Some(candidate);
        }
        if current == root {
            return None;
        }
        current = current.parent()?.to_path_buf();
    }
}

#[cfg(test)]
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

/// A semantic `Package.resolved` delta starts at its owning manifest. The
/// manifest-to-resolved edge is still represented in each reason so formatting-only
/// lockfile edits never become graph seeds.
pub(super) fn swift_resolved_seed_candidates(
    root: &Path,
    seeds: &[(PathBuf, PathBuf)],
    graph: &DepGraph,
    all_test_set: &HashSet<PathBuf>,
) -> Vec<SelectedTest> {
    let mut candidates = BTreeMap::new();
    for (resolved, manifest) in seeds {
        let (reachable_tests, path_parents) =
            bfs_path_find_set(graph, &NodeId::file(manifest.clone()), all_test_set);
        for (test_node, edge_path) in reachable_tests {
            let NodeId::File(test_path) = &test_node else {
                continue;
            };
            let rel_test = relative_path(root, test_path);
            let mut node_chain = Vec::new();
            let mut current = test_node.clone();
            node_chain.push(slash_node_name(&current, root));
            while let Some((parent, _)) = path_parents.get(&current) {
                node_chain.push(slash_node_name(parent, root));
                current = parent.clone();
            }
            node_chain.reverse();
            node_chain.insert(0, relative_path(root, resolved));
            let mut via = vec!["swift package dependency".to_string()];
            via.extend(
                edge_path
                    .iter()
                    .map(|kind| impact_reason_label(*kind).to_string()),
            );
            let mut via_details = vec![None];
            via_details.extend(via_details_from_edges(&edge_path));
            let reason = ImpactReason {
                changed_file: relative_path(root, resolved),
                path: node_chain,
                via,
                via_details,
            };
            let entry = candidates
                .entry(rel_test.clone())
                .or_insert_with(|| SelectedTest {
                    test_file: rel_test,
                    confidence: path_confidence(&edge_path),
                    targets: Vec::new(),
                    reasons: Vec::new(),
                });
            if !entry.reasons.contains(&reason) {
                entry.reasons.push(reason);
            }
        }
    }
    candidates.into_values().collect()
}

pub(super) fn swift_manifest_seed_candidates(
    root: &Path,
    manifests: &[PathBuf],
    graph: &DepGraph,
    all_test_set: &HashSet<PathBuf>,
) -> Vec<SelectedTest> {
    let mut candidates = BTreeMap::new();
    for manifest in manifests {
        let (tests, parents) =
            bfs_path_find_set(graph, &NodeId::file(manifest.clone()), all_test_set);
        for (node, edges) in tests {
            let NodeId::File(test_path) = node else {
                continue;
            };
            let mut path = vec![slash_node_name(&NodeId::file(test_path.clone()), root)];
            let mut current = NodeId::file(test_path.clone());
            while let Some((parent, _)) = parents.get(&current) {
                path.push(slash_node_name(parent, root));
                current = parent.clone();
            }
            path.reverse();
            let test_file = relative_path(root, &test_path);
            let reason = ImpactReason {
                changed_file: relative_path(root, manifest),
                path,
                via: edges
                    .iter()
                    .map(|edge| impact_reason_label(*edge).to_string())
                    .collect(),
                via_details: via_details_from_edges(&edges),
            };
            let entry = candidates
                .entry(test_file.clone())
                .or_insert_with(|| SelectedTest {
                    test_file,
                    confidence: path_confidence(&edges),
                    targets: Vec::new(),
                    reasons: Vec::new(),
                });
            if !entry.reasons.contains(&reason) {
                entry.reasons.push(reason);
            }
        }
    }
    candidates.into_values().collect()
}

#[path = "lockfile_seeds/merge.rs"]
mod merge;
pub(super) use merge::merge_lockfile_seed_candidates;

#[path = "lockfile_seeds/apply.rs"]
mod apply;
pub(super) use apply::apply_lockfile_seeds;

#[path = "lockfile_seeds/package_scope.rs"]
mod package_scope;
