use super::*;

/// A semantic `Package.resolved` delta starts at its owning manifest. The
/// manifest-to-resolved edge is still represented in each reason so formatting-only
/// lockfile edits never become graph seeds.
pub(in crate::tests::configured_plan) fn swift_resolved_seed_candidates(
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

pub(in crate::tests::configured_plan) fn swift_manifest_seed_candidates(
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
