/// Match static client route references against the configured route definitions
/// already expanded from server facts. This produces client -> route edges once;
/// route reports and related traversal only project the prepared index.
fn client_call_relationships(
    _route_facts: &HashMap<PathBuf, FileFacts>,
    all_facts: &crate::codebase::ts_source::facts::TsFactMap,
    routes: &[ServerRoute],
) -> Vec<RelationshipEdge> {
    all_facts
        .iter()
        .flat_map(|(path, file_facts)| {
            file_facts.route_refs.iter().flat_map(move |reference| {
                routes
                    .iter()
                    .filter(move |route| {
                        crate::codebase::ts_routes::matcher::matches(
                            &reference.pattern,
                            &route.route,
                        )
                    })
                    .map(move |route| RelationshipEdge {
                            from: RelationshipNode::File(path.clone()),
                            to: RelationshipNode::Route(route.route.clone()),
                            kind: EdgeKind::ClientCall,
                        })
            })
        })
        .collect()
}
