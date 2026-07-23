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
                .chain(vitest_edges.iter().flat_map(|edge| {
                    [edge.from.clone(), edge.to.clone()]
                }))
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

    pub(crate) fn contains_file(&self, path: &Path) -> bool {
        self.traversal_edges().forward().contains_key(&NodeId::File(
            crate::codebase::ts_resolver::normalize_path(path),
        ))
    }
    /// Get the direct dependents (reverse edges) of a single node.
    pub fn dependents_of_node(&self, node: &NodeId) -> Option<&Vec<(NodeId, EdgeKind)>> {
        self.traversal_edges().reverse().get(node)
    }

    /// Get the direct dependencies (forward edges) of a single node.
    pub fn dependencies_of_node(&self, node: &NodeId) -> Option<&Vec<(NodeId, EdgeKind)>> {
        self.traversal_edges().forward().get(node)
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
        file_universe: &HashSet<PathBuf>,
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
        bfs_skipping_symbol_owner_files(&roots, self.traversal_edges().reverse(), max_depth, allowed)
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
        let mut visited_pairs: HashSet<(PathBuf, String)> = HashSet::new();
        let mut queue: VecDeque<(PathBuf, String)> = VecDeque::new();
        let mut direct_importers: HashSet<NodeId> = HashSet::new();

        let start = (
            crate::codebase::ts_resolver::normalize_path(file),
            symbol.to_string(),
        );
        visited_pairs.insert(start.clone());
        queue.push_back(start);

        while let Some((src_file, sym)) = queue.pop_front() {
            if let Some(importers) = symbol_index.importers_of(&src_file, &sym) {
                for (importer, local_name, is_reexport) in importers {
                    direct_importers.insert(NodeId::File(importer.clone()));
                    if *is_reexport {
                        let pair = (importer.clone(), local_name.clone());
                        push_unvisited_symbol_pair(&mut visited_pairs, &mut queue, pair);
                    }
                }
            }
        }

        // Also check if (file, symbol) corresponds to a QueueJob node.
        let queue_job = NodeId::QueueJob {
            queue_file: file.to_path_buf(),
            job: symbol.to_string(),
        };
        if self.traversal_edges().reverse().contains_key(&queue_job) {
            direct_importers.insert(queue_job);
        }

        if max_depth == Some(0) {
            return Vec::new();
        }

        let mut roots: Vec<NodeId> = direct_importers.into_iter().collect();
        roots.sort();
        let mut entries = roots
            .iter()
            .cloned()
            .map(|node| NodeEntry {
                node,
                depth: 1,
                via: vec![EdgeKind::Import],
            })
            .collect::<Vec<_>>();
        let remaining_depth = max_depth.map(|depth| depth.saturating_sub(1));
        let mut downstream = bfs(&roots, self.traversal_edges().reverse(), remaining_depth, allowed);
        for entry in &mut downstream {
            entry.depth += 1;
        }
        entries.extend(downstream);
        entries
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

    fn merge_canonical_edges(&mut self, edges: Vec<Edge>) {
        let current = std::mem::take(&mut self.edges);
        let nodes = current.forward().keys().cloned().collect::<Vec<_>>();
        let combined = current
            .edges()
            .iter()
            .cloned()
            .chain(edges.into_iter().map(|(from, to, kind)| {
                CanonicalEdge::new(from, to, kind)
            }));
        self.edges = EdgeIndex::from_edges_and_nodes(combined, nodes);
        sort_edge_index_adjacency(&mut self.edges);
        self.effective_edges = OnceLock::new();
    }
}
