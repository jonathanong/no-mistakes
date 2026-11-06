impl DepGraph {
    pub(crate) fn call_source_nodes_with_root_mocks(
        &self,
        roots: &[NodeId],
        max_depth: Option<usize>,
    ) -> FxHashSet<NodeId> {
        if !roots
            .iter()
            .filter_map(NodeId::as_file)
            .any(|file| self.root_mock_boundaries.contains_key(file))
        {
            return self.call_source_nodes(roots, CallTraversal::Transitive, max_depth);
        }
        let mut groups: FxHashMap<Option<&Path>, Vec<NodeId>> = fx_map();
        for root in roots {
            groups.entry(root.as_file()).or_default().push(root.clone());
        }
        let mut selected = fx_set();
        let mut ordinary = Vec::new();
        for (file, roots) in groups {
            let Some(boundaries) = file.and_then(|file| self.root_mock_boundaries.get(file)) else {
                ordinary.extend(roots);
                continue;
            };
            if boundaries.blocked.is_empty() {
                ordinary.extend(roots);
                ordinary.extend(boundaries.factories.iter().cloned());
                continue;
            }
            selected.extend(self.walk_with_mock_boundaries(&roots, max_depth, boundaries));
        }
        selected.extend(self.call_source_nodes(&ordinary, CallTraversal::Transitive, max_depth));
        selected
    }

    fn walk_with_mock_boundaries(
        &self,
        roots: &[NodeId],
        max_depth: Option<usize>,
        boundaries: &RootMockBoundaries,
    ) -> FxHashSet<NodeId> {
        let mut nodes: FxHashSet<NodeId> = roots
            .iter()
            .chain(&boundaries.factories)
            .filter(|node| {
                !node
                    .as_file()
                    .is_some_and(|file| boundaries.blocked.contains(file))
            })
            .cloned()
            .collect();
        let mut queue: VecDeque<_> = nodes.iter().cloned().map(|node| (node, 0usize)).collect();
        let cap = max_depth.unwrap_or(usize::MAX);
        let edges = self.traversal_edges().forward();
        while let Some((node, depth)) = queue.pop_front() {
            if depth >= cap.saturating_sub(1) {
                continue;
            }
            let Some(neighbors) = edges.get(&node) else {
                continue;
            };
            for (neighbor, kind) in &neighbors.neighbors {
                if *kind == EdgeKind::Call
                    && !neighbor
                        .as_file()
                        .is_some_and(|file| boundaries.blocked.contains(file))
                    && nodes.insert(neighbor.clone())
                {
                    queue.push_back((neighbor.clone(), depth + 1));
                }
            }
        }
        nodes
    }
}

impl DepGraph {
    pub(crate) fn runtime_import_sources_with_root_mocks(
        &self,
        roots: &[NodeId],
        max_depth: Option<usize>,
    ) -> std::collections::BTreeSet<PathBuf> {
        let mut groups: FxHashMap<Vec<&PathBuf>, Vec<NodeId>> = fx_map();
        for root in roots {
            let Some(file) = root.as_file() else { continue };
            let mut blocked: Vec<_> = self
                .root_mock_boundaries
                .get(file)
                .into_iter()
                .flat_map(|value| value.blocked.iter())
                .collect();
            blocked.sort_unstable();
            groups.entry(blocked).or_default().push(NodeId::file(file));
        }
        let observer = crate::diagnostics::current();
        groups
            .into_par_iter()
            .map(|(blocked, roots)| {
                let mut files = std::collections::BTreeSet::new();
                let blocked: FxHashSet<_> = blocked.into_iter().map(PathBuf::as_path).collect();
                let mut seen = fx_set();
                let mut queue = VecDeque::new();
                for node in roots {
                    if !node.as_file().is_some_and(|file| blocked.contains(file))
                        && seen.insert(node.clone())
                    {
                        queue.push_back((node, 0usize));
                    }
                }
                while let Some((node, depth)) = queue.pop_front() {
                    let Some(file) = node.as_file() else { continue };
                    if let Some(observer) = &observer {
                        observer.increment("graph.runtime_import_visits", 1);
                    }
                    files.insert(file.to_path_buf());
                    if max_depth.is_some_and(|cap| depth >= cap.saturating_sub(1)) {
                        continue;
                    }
                    if let Some(neighbors) = self.dependencies_of_node(&node) {
                        for (node, kind) in neighbors {
                            if matches!(
                                kind,
                                EdgeKind::Import
                                    | EdgeKind::WorkspaceImport
                                    | EdgeKind::Require
                                    | EdgeKind::DynamicImport
                                    | EdgeKind::ConditionalDynamicImport
                            ) && !node.as_file().is_some_and(|file| blocked.contains(file))
                                && seen.insert(node.clone())
                            {
                                queue.push_back((node.clone(), depth + 1));
                            }
                        }
                    }
                }
                files
            })
            .reduce(std::collections::BTreeSet::new, |mut files, group| {
                files.extend(group);
                files
            })
    }
}

#[cfg(test)]
#[path = "root_mocks_walk_tests.rs"]
mod runtime_import_walk_tests;
