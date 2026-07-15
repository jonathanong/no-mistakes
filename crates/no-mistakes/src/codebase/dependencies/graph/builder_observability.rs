fn record_graph_observability(
    graph: &DepGraph,
    session: &crate::codebase::analysis_session::AnalysisSession,
) {
    let Some(observer) = session.observer().filter(|observer| observer.verbose()) else {
        return;
    };
    let mut nodes = HashSet::new();
    let mut edges = 0_u64;
    for (from, neighbors) in graph.edges.forward() {
        nodes.insert(from);
        edges += neighbors.len() as u64;
        nodes.extend(neighbors.iter().map(|(to, _)| to));
    }
    observer.increment("graph.nodes", nodes.len() as u64);
    observer.increment("graph.edges", edges);
}
