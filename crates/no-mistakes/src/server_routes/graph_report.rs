pub(super) fn build_report(
    root: &Path,
    facts: &HashMap<PathBuf, FileFacts>,
    tsconfig: &TsConfig,
) -> ProjectReport {
    build_report_and_relationships(root, facts, tsconfig).0
}

pub(super) fn build_prepared_report(
    root: &Path,
    facts: &HashMap<PathBuf, FileFacts>,
    tsconfig: &TsConfig,
) -> PreparedProjectReport {
    let (report, mut relationships) = build_report_and_relationships(root, facts, tsconfig);
    relationships.sort_by_key(|edge| {
        (
            public_node(root, &edge.from),
            public_node(root, &edge.to),
            edge.kind,
        )
    });
    relationships.dedup();
    let mut nodes_by_name = HashMap::<String, Vec<RelationshipNode>>::new();
    for relationship in &relationships {
        for node in [&relationship.from, &relationship.to] {
            let nodes = nodes_by_name.entry(public_node(root, node)).or_default();
            if !nodes.contains(node) {
                nodes.push(node.clone());
            }
        }
    }
    for nodes in nodes_by_name.values_mut() {
        nodes.sort();
    }
    let aliases = NodeAliases::from_groups(nodes_by_name.values().cloned());
    let index = EdgeIndex::from_edges(
        relationships
            .into_iter()
            .map(|edge| CanonicalEdge::new(edge.from, edge.to, edge.kind)),
    );
    PreparedProjectReport {
        root: root.to_path_buf(),
        report,
        index,
        nodes_by_name,
        aliases,
    }
}

fn build_report_and_relationships(
    root: &Path,
    facts: &HashMap<PathBuf, FileFacts>,
    tsconfig: &TsConfig,
) -> (ProjectReport, Vec<RelationshipEdge>) {
    let mut routes = Vec::new();
    let mut edges = Vec::new();
    let mut relationships = Vec::new();
    let mut diagnostics = Vec::new();
    let visible = facts.keys().cloned().collect::<HashSet<_>>();
    let resolver = ImportResolver::new(tsconfig).with_visible(&visible);
    let mounts = resolve_mounts_with_resolver(facts, &resolver);
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
