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
