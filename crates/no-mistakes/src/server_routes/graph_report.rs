pub(super) fn build_report(
    root: &Path,
    facts: &HashMap<PathBuf, FileFacts>,
    tsconfig: &TsConfig,
) -> ProjectReport {
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    build_report_with_session(root, facts, &Default::default(), tsconfig, &session)
}

fn build_report_with_session(
    root: &Path,
    facts: &HashMap<PathBuf, FileFacts>,
    all_facts: &crate::codebase::ts_source::facts::TsFactMap,
    tsconfig: &TsConfig,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> ProjectReport {
    let visible = facts.keys().cloned().collect::<HashSet<_>>();
    let resolver = ImportResolver::new_in_session(tsconfig, Some(&visible), session);
    build_report_with_resolver(root, facts, all_facts, &resolver)
}

fn build_report_with_resolver(
    root: &Path,
    facts: &HashMap<PathBuf, FileFacts>,
    all_facts: &crate::codebase::ts_source::facts::TsFactMap,
    resolver: &dyn ImportResolution,
) -> ProjectReport {
    build_report_and_relationships(root, facts, all_facts, resolver).0
}

pub(super) fn build_prepared_report(
    root: &Path,
    facts: &HashMap<PathBuf, FileFacts>,
    all_facts: &crate::codebase::ts_source::facts::TsFactMap,
    tsconfig: &TsConfig,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> PreparedProjectReport {
    let visible = facts.keys().cloned().collect::<HashSet<_>>();
    let resolver = ImportResolver::new_in_session(tsconfig, Some(&visible), session);
    let (report, relationships) = build_report_and_relationships(root, facts, all_facts, &resolver);
    PreparedProjectReport {
        report,
        relationships: PreparedRelationshipIndex::from_edges(
            relationships
                .into_iter()
                .map(|edge| CanonicalEdge::new(edge.from, edge.to, edge.kind)),
            |node| public_node(root, node),
        ),
    }
}

fn build_report_and_relationships(
    root: &Path,
    facts: &HashMap<PathBuf, FileFacts>,
    all_facts: &crate::codebase::ts_source::facts::TsFactMap,
    resolver: &dyn ImportResolution,
) -> (ProjectReport, Vec<RelationshipEdge>) {
    let mut routes = Vec::new();
    let mut relationships = Vec::new();
    let mut diagnostics = Vec::new();
    let mounts = resolve_mounts_with_resolver(facts, resolver);
    for (path, file_facts) in facts {
        diagnostics.extend(
            file_facts
                .diagnostics
                .iter()
                .map(|(line, message)| Diagnostic {
                    severity: Severity::Warning,
                    file: relative_string(root, path),
                    line: *line,
                    message: message.clone(),
                }),
        );
        for site in &file_facts.routes {
            for route in expand_site(root, site, facts, &mounts) {
                let relationship = RelationshipEdge {
                    from: RelationshipNode::File(root.join(&route.file)),
                    to: RelationshipNode::Route(route.route.clone()),
                    kind: EdgeKind::ServerRoute,
                };
                relationships.push(relationship);
                routes.push(route);
            }
        }
    }
    routes.sort();
    routes.dedup();
    relationships.extend(client_call_relationships(facts, all_facts, &routes));
    relationships.sort();
    relationships.dedup();
    let mut edges = relationships
        .iter()
        .map(|relationship| Edge {
            from: public_node(root, &relationship.from),
            to: public_node(root, &relationship.to),
            kind: relationship.kind,
        })
        .collect::<Vec<_>>();
    edges.sort();
    edges.dedup();
    diagnostics.sort();
    diagnostics.dedup();
    let dynamic_routes = routes
        .iter()
        .filter(|route| route.route.contains('*'))
        .count();
    (ProjectReport {
        summary: Summary {
            total_routes: routes.len(),
            total_files: facts.len(),
            dynamic_routes,
        },
        routes,
        edges,
        diagnostics,
    }, relationships)
}

include!("graph_client_calls.rs");

pub(crate) fn public_node(root: &Path, node: &RelationshipNode) -> String {
    match node {
        RelationshipNode::File(file) => relative_string(root, file),
        RelationshipNode::Route(route) => route.clone(),
    }
}

fn expand_site(
    root: &Path,
    site: &RouteSite,
    facts: &HashMap<PathBuf, FileFacts>,
    mounts: &[crate::server_routes::mounts::ResolvedMount],
) -> Vec<ServerRoute> {
    prefixes_for(site, facts, mounts)
        .into_iter()
        .map(|prefix| {
            let raw_path = join_paths(&prefix, &site.raw_path);
            ServerRoute {
                file: relative_string(root, &site.file),
                line: site.line,
                method: site.method.clone(),
                route: normalize_route(&raw_path),
                raw_path,
                query_params: site.query_params.clone(),
                framework: site.framework,
            }
        })
        .collect()
}
