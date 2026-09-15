impl DepGraph {
    /// Import-only adjacency from one lazy reachable walk. Not the canonical
    /// full-universe graph: only files the union of import roots actually reached.
    pub(crate) fn from_import_edges(
        root: PathBuf,
        edges: Vec<CanonicalEdge<NodeId, EdgeKind>>,
        nodes: impl IntoIterator<Item = NodeId>,
        session: &crate::codebase::analysis_session::AnalysisSession,
    ) -> Self {
        session.record_work("graph.builds", 1);
        let graph = Self {
            root,
            edges: EdgeIndex::from_edges_and_nodes(edges, nodes),
            callable_nodes_by_file: fx_map(),
            callable_export_resolutions: fx_map(),
            resolved_call_sites: Vec::new(),
            call_sites_by_file: fx_map(),
            vitest_setup_projects: Vec::new(),
            effective_edges: OnceLock::new(),
            parse_errors: HashMap::new(),
            resource_edge_details: fx_map(),
            resource_diagnostics: Vec::new(),
        };
        record_graph_observability(&graph, session);
        graph
    }
}
