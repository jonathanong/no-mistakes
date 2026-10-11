impl DepGraph {
    pub(crate) fn deps_of_excluding_files(
        &self,
        roots: &[NodeId],
        allowed: Option<&HashSet<EdgeKind>>,
        excluded: &HashSet<PathBuf>,
    ) -> Vec<NodeEntry> {
        let roots = normalize_nodes(roots);
        bfs_excluding_files(&roots, self.traversal_edges().forward(), None, allowed, excluded)
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
