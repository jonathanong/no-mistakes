impl DepGraph {
    /// Get the direct dependents (reverse edges) of a single node.
    pub fn dependents_of_node(&self, node: &NodeId) -> Option<&Vec<(NodeId, EdgeKind)>> {
        self.reverse.get(node)
    }

    /// Get the direct dependencies (forward edges) of a single node.
    pub fn dependencies_of_node(&self, node: &NodeId) -> Option<&Vec<(NodeId, EdgeKind)>> {
        self.forward.get(node)
    }

    /// Find all nodes that `roots` transitively depend on (follow imports).
    pub fn deps_of(
        &self,
        roots: &[NodeId],
        max_depth: Option<usize>,
        allowed: Option<&HashSet<EdgeKind>>,
    ) -> Vec<NodeEntry> {
        let roots = normalize_nodes(roots);
        bfs(&roots, &self.forward, max_depth, allowed)
    }

    /// Find all nodes that transitively reference `roots` (reverse direction).
    pub fn dependents_of(
        &self,
        roots: &[NodeId],
        max_depth: Option<usize>,
        allowed: Option<&HashSet<EdgeKind>>,
    ) -> Vec<NodeEntry> {
        let roots = normalize_nodes(roots);
        bfs(&roots, &self.reverse, max_depth, allowed)
    }

    /// Reverse traversal for symbol roots. The graph includes file -> symbol
    /// bridge edges so file-level roots can expose exported symbols, but a
    /// symbol root must not immediately widen back to its owning file.
    pub fn dependents_of_symbol_nodes(
        &self,
        roots: &[NodeId],
        max_depth: Option<usize>,
        allowed: Option<&HashSet<EdgeKind>>,
    ) -> Vec<NodeEntry> {
        let roots = normalize_nodes(roots);
        bfs_skipping_initial_symbol_owner_files(&roots, &self.reverse, max_depth, allowed)
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
        if self.reverse.contains_key(&queue_job) {
            direct_importers.insert(queue_job);
        }

        let roots: Vec<NodeId> = direct_importers.into_iter().collect();
        bfs(&roots, &self.reverse, max_depth, allowed)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn all_files(&self) -> impl Iterator<Item = &NodeId> {
        self.forward.keys()
    }
}
