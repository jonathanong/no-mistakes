impl DepGraph {
    /// Find symbol importers while pruning excluded importers and reverse bridges.
    pub(crate) fn dependents_of_symbol_excluding_files(
        &self,
        file: &Path,
        symbol: &str,
        max_depth: Option<usize>,
        allowed: Option<&HashSet<EdgeKind>>,
        symbol_index: &SymbolIndex,
        excluded: &HashSet<PathBuf>,
    ) -> Vec<NodeEntry> {
        let mut visited_pairs: FxHashSet<(Arc<Path>, Arc<str>)> = fx_set();
        let mut queue: VecDeque<(Arc<Path>, Arc<str>)> = VecDeque::new();
        let mut direct_importers = HashSet::new();
        let start = (intern_node_path(file), intern_node_str(symbol));
        visited_pairs.insert(start.clone());
        queue.push_back(start);
        while let Some((src_file, sym)) = queue.pop_front() {
            if let Some(importers) = symbol_index.importers_of(&src_file, &sym) {
                for (importer, local_name, is_reexport) in importers {
                    if excluded.contains(importer.as_ref()) {
                        continue;
                    }
                    direct_importers.insert(NodeId::file(importer.as_ref()));
                    if *is_reexport {
                        let pair = (Arc::clone(importer), Arc::clone(local_name));
                        push_unvisited_symbol_pair(&mut visited_pairs, &mut queue, pair);
                    }
                }
            }
        }
        let queue_job = NodeId::queue_job(file, symbol);
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
        let mut downstream = bfs_excluding_files(
            &roots,
            self.traversal_edges().reverse(),
            remaining_depth,
            allowed,
            excluded,
        );
        for entry in &mut downstream {
            entry.depth += 1;
        }
        entries.extend(downstream);
        entries
    }

    /// Follow reverse edges while treating excluded files as traversal barriers.
    pub(crate) fn dependents_of_excluding_files(
        &self,
        roots: &[NodeId],
        max_depth: Option<usize>,
        allowed: Option<&HashSet<EdgeKind>>,
        excluded: &HashSet<PathBuf>,
    ) -> Vec<NodeEntry> {
        let roots = normalize_nodes(roots);
        bfs_excluding_files(
            &roots,
            self.traversal_edges().reverse(),
            max_depth,
            allowed,
            excluded,
        )
    }

    /// Reverse symbol traversal with excluded owner/importer files pruned in BFS.
    pub(crate) fn dependents_of_symbol_nodes_excluding_files(
        &self,
        roots: &[NodeId],
        max_depth: Option<usize>,
        allowed: Option<&HashSet<EdgeKind>>,
        excluded: &HashSet<PathBuf>,
    ) -> Vec<NodeEntry> {
        let roots = normalize_nodes(roots);
        bfs_skipping_symbol_owner_files_excluding_files(
            &roots,
            self.traversal_edges().reverse(),
            max_depth,
            allowed,
            excluded,
        )
    }

    pub(crate) fn deps_of_excluding_files(
        &self,
        roots: &[NodeId],
        allowed: Option<&HashSet<EdgeKind>>,
        excluded: &HashSet<PathBuf>,
    ) -> Vec<NodeEntry> {
        let roots = normalize_nodes(roots);
        bfs_excluding_files(
            &roots,
            self.traversal_edges().forward(),
            None,
            allowed,
            excluded,
        )
    }

    pub(crate) fn deps_of_in_file_universe_excluding_files(
        &self,
        roots: &[NodeId],
        allowed: Option<&HashSet<EdgeKind>>,
        file_universe: &crate::fx::PathSet,
        excluded: &HashSet<PathBuf>,
    ) -> Vec<NodeEntry> {
        let roots = normalize_nodes(roots);
        bfs_in_file_universe_excluding_files(
            &roots,
            self.traversal_edges().forward(),
            None,
            allowed,
            file_universe,
            excluded,
        )
    }
}
