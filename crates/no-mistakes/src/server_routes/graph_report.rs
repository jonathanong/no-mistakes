pub(super) fn build_report(
    root: &Path,
    facts: &HashMap<PathBuf, FileFacts>,
    tsconfig: &TsConfig,
) -> ProjectReport {
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    build_report_with_session(root, facts, tsconfig, &session)
}

fn build_report_with_session(
    root: &Path,
    facts: &HashMap<PathBuf, FileFacts>,
    tsconfig: &TsConfig,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> ProjectReport {
    let visible = facts.keys().cloned().collect::<HashSet<_>>();
    let resolver = ImportResolver::new_in_session(tsconfig, Some(&visible), session);
    build_report_with_resolver(root, facts, &resolver)
}

fn build_report_with_resolver(
    root: &Path,
    facts: &HashMap<PathBuf, FileFacts>,
    resolver: &dyn ImportResolution,
) -> ProjectReport {
    build_report_and_relationships(root, facts, resolver).0
}

pub(super) fn build_prepared_report(
    root: &Path,
    facts: &HashMap<PathBuf, FileFacts>,
    tsconfig: &TsConfig,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> PreparedProjectReport {
    let visible = facts.keys().cloned().collect::<HashSet<_>>();
    let resolver = ImportResolver::new_in_session(tsconfig, Some(&visible), session);
    let (report, relationships) = build_report_and_relationships(root, facts, &resolver);
    let relationships = PreparedRelationshipIndex::from_edges(
        relationships
            .into_iter()
            .map(|edge| CanonicalEdge::new(edge.from, edge.to, edge.kind)),
        |node| public_node(root, node),
    );
    PreparedProjectReport {
        report,
        relationships,
    }
}

fn build_report_and_relationships(
    root: &Path,
    facts: &HashMap<PathBuf, FileFacts>,
    resolver: &dyn ImportResolution,
) -> (ProjectReport, Vec<RelationshipEdge>) {
    let mut routes = Vec::new();
    let mut edges = Vec::new();
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
                edges.push(Edge {
                    from: public_node(root, &relationship.from),
                    to: public_node(root, &relationship.to),
                    kind: relationship.kind,
                });
                relationships.push(relationship);
                routes.push(route);
            }
        }
    }
    routes.sort();
    routes.dedup();
    edges.sort();
    edges.dedup();
    diagnostics.sort();
    diagnostics.dedup();
    let dynamic_routes = routes
        .iter()
        .filter(|route| route.route.contains('*'))
        .count();
    let report = ProjectReport {
        summary: Summary {
            total_routes: routes.len(),
            total_files: facts.len(),
            dynamic_routes,
        },
        routes,
        edges,
        diagnostics,
    };
    (report, relationships)
}

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
