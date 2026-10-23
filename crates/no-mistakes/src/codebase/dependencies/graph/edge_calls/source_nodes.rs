impl DepGraph {
    /// Select call-site owners from roots already returned by `expand_call_roots`.
    /// Unlike trace queries, policies need the union of reachable sources, with
    /// each source strictly before the final permitted call hop.
    pub(crate) fn call_source_nodes(
        &self,
        roots: &[NodeId],
        traversal: CallTraversal,
        max_depth: Option<usize>,
    ) -> FxHashSet<NodeId> {
        let mut nodes: FxHashSet<NodeId> = roots.iter().cloned().collect();
        let cap = max_depth.unwrap_or(usize::MAX);
        if traversal != CallTraversal::Transitive || cap <= 1 || nodes.is_empty() {
            return nodes;
        }
        let edges = self.traversal_edges().forward();
        let mut queue = nodes
            .iter()
            .cloned()
            .map(|node| (node, 0usize))
            .collect::<std::collections::VecDeque<_>>();
        let mut expanded = 0u64;
        while let Some((node, depth)) = queue.pop_front() {
            // Nodes at cap - 1 own eligible call sites, but their callees cannot.
            if depth >= cap - 1 {
                continue;
            }
            expanded += 1;
            let Some(neighbors) = edges.get(&node) else {
                continue;
            };
            for (neighbor, kind) in &neighbors.neighbors {
                if *kind == EdgeKind::Call && nodes.insert(neighbor.clone()) {
                    queue.push_back((neighbor.clone(), depth + 1));
                }
            }
        }
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment("graph.call_source_nodes.expanded", expanded);
        }
        nodes
    }
}
