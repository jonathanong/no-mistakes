use super::*;

pub(in crate::tests::configured_plan) fn dotnet_dependency_seed_candidates(
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
                        Some(graph),
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

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "../lockfile_seeds_dotnet_semantic_seed_tests.rs"]
mod dotnet_semantic_seed_tests;

pub(in crate::tests::configured_plan) fn central_package_seed_roots(
    root: &Path,
    props: &Path,
    packages: &std::collections::BTreeSet<String>,
    facts: &no_mistakes::codebase::dotnet::DotnetFactMap,
    visible_paths: &[PathBuf],
    graph: Option<&DepGraph>,
) -> Vec<PathBuf> {
    facts
        .projects
        .values()
        .filter(|project| {
            project.package_references.iter().any(|reference| {
                packages
                    .iter()
                    .any(|package| reference.eq_ignore_ascii_case(package))
            }) && nearest_central_props(root, &project.project_dir, visible_paths)
                .is_some_and(|nearest| central_import_chain_contains(&nearest, props, graph))
        })
        .map(|project| project.project_path.clone())
        .collect()
}

fn central_import_chain_contains(central: &Path, sought: &Path, graph: Option<&DepGraph>) -> bool {
    central == sought
        || graph.is_some_and(|graph| {
            graph
            .deps_of(
                &[NodeId::file(central)],
                None,
                Some(&HashSet::from([
                    no_mistakes::codebase::dependencies::graph::EdgeKind::DotnetProjectDependency,
                ])),
            )
            .iter()
            .any(|entry| entry.node.as_file() == Some(sought))
        })
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
