/// Match configured client sources against route definitions already expanded
/// from server facts. This produces client -> route edges once; route reports
/// and related traversal only project the prepared index.
fn client_call_relationships(
    facts: &crate::codebase::ts_source::facts::TsFactMap,
    tsconfig: &TsConfig,
    session: &crate::codebase::analysis_session::AnalysisSession,
    routes: &[ServerRoute],
) -> Vec<RelationshipEdge> {
    let visible = facts.keys().cloned().collect::<HashSet<_>>();
    let graph_files = crate::codebase::dependencies::graph::GraphFiles::from_files(
        visible.iter().cloned().collect(),
    );
    let resolver = ImportResolver::new_in_session(tsconfig, Some(&visible), session);
    facts
        .iter()
        .flat_map(|(path, file_facts)| {
            let mut references: Vec<_> = file_facts
                .route_refs
                .iter()
                .map(|reference| reference.pattern.clone())
                .collect();
            references.extend(
                crate::codebase::dependencies::graph::route_helper_ref_patterns_with_lines(
                    path,
                    file_facts,
                    facts,
                    &resolver,
                    &graph_files,
                )
                .into_iter()
                .map(|(_, pattern)| pattern),
            );
            references.into_iter().flat_map(move |reference| {
                routes
                    .iter()
                    .filter(move |route| {
                        crate::codebase::ts_routes::matcher::matches(
                            &reference,
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
