impl DepGraph {
    /// Return dynamic resource calls that intentionally produced no graph edge.
    pub fn resource_diagnostics(&self) -> &[ResourceGraphDiagnostic] {
        &self.resource_diagnostics
    }

    /// Return every static call site that created the `consumer → resource`
    /// edge, in deterministic call-site order.
    pub fn resource_edge_details(
        &self,
        consumer: &Path,
        resource: &Path,
    ) -> Option<&[ResourceCallSite]> {
        let key = (
            crate::codebase::ts_resolver::normalize_path(consumer),
            crate::codebase::ts_resolver::normalize_path(resource),
        );
        self.resource_edge_details.get(&key).map(Vec::as_slice)
    }

    fn traversal_edges(&self) -> &EdgeIndex<NodeId, EdgeKind> {
        if self.vitest_setup_projects.is_empty() {
            return &self.edges;
        }
        self.effective_edges.get_or_init(|| {
            let vitest_edges = self.materialize_vitest_setup_edges();
            let nodes = self
                .edges
                .forward()
                .keys()
                .cloned()
                .chain(
                    vitest_edges
                        .iter()
                        .flat_map(|edge| [edge.from.clone(), edge.to.clone()]),
                )
                .collect::<Vec<_>>();
            let edges = self.edges.edges().iter().cloned().chain(vitest_edges);
            let mut index = EdgeIndex::from_edges_and_nodes(edges, nodes);
            sort_edge_index_adjacency(&mut index);
            index
        })
    }

    pub(crate) fn parse_error(&self, path: &Path) -> Option<&str> {
        self.parse_errors.get(path).map(String::as_str)
    }

    /// Every file whose facts could not be collected, with the reason. The
    /// order is unspecified, so a caller that reports these must sort.
    pub(crate) fn parse_errors(&self) -> impl Iterator<Item = (&Path, &str)> {
        self.parse_errors
            .iter()
            .map(|(path, error)| (path.as_path(), error.as_str()))
    }

    pub(crate) fn contains_file(&self, path: &Path) -> bool {
        self.traversal_edges().forward().contains_key(&NodeId::file(
            crate::codebase::ts_resolver::normalize_path(path),
        ))
    }
    /// Get the direct dependents (reverse edges) of a single node.
    pub fn dependents_of_node(&self, node: &NodeId) -> Option<&Vec<(NodeId, EdgeKind)>> {
        self.traversal_edges()
            .reverse()
            .get(node)
            .map(|adj| &adj.neighbors)
    }

    /// Get the direct dependencies (forward edges) of a single node.
    pub fn dependencies_of_node(&self, node: &NodeId) -> Option<&Vec<(NodeId, EdgeKind)>> {
        self.traversal_edges()
            .forward()
            .get(node)
            .map(|adj| &adj.neighbors)
    }

    /// Find all nodes that `roots` transitively depend on (follow imports).
    pub fn deps_of(
        &self,
        roots: &[NodeId],
        max_depth: Option<usize>,
        allowed: Option<&HashSet<EdgeKind>>,
    ) -> Vec<NodeEntry> {
        let roots = normalize_nodes(roots);
        bfs(&roots, self.traversal_edges().forward(), max_depth, allowed)
    }

    pub(crate) fn deps_of_in_file_universe(
        &self,
        roots: &[NodeId],
        max_depth: Option<usize>,
        allowed: Option<&HashSet<EdgeKind>>,
        file_universe: &crate::fx::PathSet,
    ) -> Vec<NodeEntry> {
        let roots = normalize_nodes(roots);
        bfs_in_file_universe(
            &roots,
            self.traversal_edges().forward(),
            max_depth,
            allowed,
            file_universe,
        )
    }

    /// Find all nodes that transitively reference `roots` (reverse direction).
    pub fn dependents_of(
        &self,
        roots: &[NodeId],
        max_depth: Option<usize>,
        allowed: Option<&HashSet<EdgeKind>>,
    ) -> Vec<NodeEntry> {
        let roots = normalize_nodes(roots);
        bfs(&roots, self.traversal_edges().reverse(), max_depth, allowed)
    }

    /// Reverse traversal for symbol roots. The graph includes file -> symbol
    /// bridge edges so file-level roots can expose exported symbols, but a
    /// symbol root must not widen back to any symbol's owning file.
    pub fn dependents_of_symbol_nodes(
        &self,
        roots: &[NodeId],
        max_depth: Option<usize>,
        allowed: Option<&HashSet<EdgeKind>>,
    ) -> Vec<NodeEntry> {
        let roots = normalize_nodes(roots);
        bfs_skipping_symbol_owner_files(
            &roots,
            self.traversal_edges().reverse(),
            max_depth,
            allowed,
        )
    }

    /// Find all files that import `symbol` from `file`, transitively.
    pub fn dependents_of_symbol(
        &self,
        file: &Path,
        symbol: &str,
        max_depth: Option<usize>,
        allowed: Option<&HashSet<EdgeKind>>,
        symbol_index: &SymbolIndex,
    ) -> Vec<NodeEntry> {
        self.dependents_of_symbol_excluding_files(
            file,
            symbol,
            max_depth,
            allowed,
            symbol_index,
            &HashSet::new(),
        )
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub(crate) fn has_reverse_node(&self, node: &NodeId) -> bool {
        self.traversal_edges().reverse().contains_key(node)
    }

    pub fn all_files(&self) -> impl Iterator<Item = &NodeId> {
        self.traversal_edges().forward().keys()
    }

    fn append_canonical_edges(&mut self, edges: Vec<Edge>) {
        if edges.is_empty() {
            return;
        }
        self.edges.extend_edges_preserving_ordinals(
            edges
                .into_iter()
                .map(|(from, to, kind)| CanonicalEdge::new(from, to, kind)),
        );
        sort_edge_index_adjacency(&mut self.edges);
        self.effective_edges = OnceLock::new();
    }
}
