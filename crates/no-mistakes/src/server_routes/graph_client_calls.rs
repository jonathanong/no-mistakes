/// Match configured client sources against route definitions already expanded
/// from server facts. This produces client -> route edges once; route reports
/// and related traversal only project the prepared index.
fn client_call_relationships(
    inputs: ClientRelationshipInputs<'_>,
    routes: &[ServerRoute],
) -> Vec<RelationshipEdge> {
    let mut relationships = Vec::new();
    for path in inputs.source_paths {
        let Some(file_facts) = inputs.facts.get(path) else {
            continue;
        };
            let mut references: Vec<_> = file_facts
                .route_refs
                .iter()
                .map(|reference| reference.pattern.clone())
                .collect();
            references.extend(
                crate::codebase::dependencies::graph::route_helper_ref_patterns_with_lines(
                    path,
                    file_facts,
                    inputs.facts,
                    &inputs.prepared.resolver,
                    &inputs.prepared.graph_files,
                )
                .into_iter()
                .map(|(_, pattern)| pattern),
            );
        for reference in references {
            relationships.extend(
                routes
                    .iter()
                    .filter(|route| {
                        crate::codebase::ts_routes::matcher::matches(&reference, &route.route)
                    })
                    .map(|route| RelationshipEdge {
                        from: RelationshipNode::File(path.clone()),
                        to: RelationshipNode::Route(route.route.clone()),
                        kind: EdgeKind::ClientCall,
                    }),
            );
        }
    }
    relationships
}
