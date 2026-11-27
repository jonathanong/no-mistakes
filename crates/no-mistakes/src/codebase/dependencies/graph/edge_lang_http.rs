fn collect_lang_http_client_edges(
    inputs: &GraphEdgeBuildInputs<'_>,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> Vec<Edge> {
    let Some(options) = inputs.config_options else {
        return Vec::new();
    };
    let config = lang_config_from_options(options);
    if config_is_empty(&config) {
        return Vec::new();
    }
    let files = inputs.graph_files.all_shared();
    let sources = graph_edge_sources(session, inputs);
    let facts = session.lang_frontend_facts(inputs.root, &files, &config, &sources);
    let mut edges = Vec::new();
    for map in [
        &facts.python,
        &facts.go,
        &facts.rust,
        &facts.ruby,
        &facts.php,
        &facts.java,
        &facts.kotlin,
        &facts.elixir,
    ] {
        edges.extend(http_edges_for_lang_map(map, &inputs.interner));
    }
    edges
}

fn http_edges_for_lang_map(
    map: &crate::codebase::lang_frontends::LangFactMap,
    interner: &PathInterner,
) -> Vec<Edge> {
    let files: Vec<HttpClientFile<'_>> = map
        .files
        .values()
        .map(|file| HttpClientFile {
            path: file.path.as_path(),
            calls: &file.http_calls,
            routes: &file.route_handlers,
        })
        .collect();
    http_edges_for_client_files(&files, interner)
}

struct HttpClientFile<'a> {
    path: &'a Path,
    calls: &'a [String],
    routes: &'a [(String, String)],
}

fn http_edges_for_client_files(files: &[HttpClientFile<'_>], interner: &PathInterner) -> Vec<Edge> {
    let mut edges = Vec::new();
    for caller in files {
        for call in caller.calls {
            for route_file in files {
                if route_file.path == caller.path {
                    continue;
                }
                let matched = route_file.routes.iter().any(|(route, _)| {
                    crate::codebase::lang_frontends::client_matches_route(call, route)
                });
                if matched {
                    edges.push((
                        NodeId::file_in(interner, caller.path),
                        NodeId::file_in(interner, route_file.path),
                        EdgeKind::HttpCall,
                    ));
                }
            }
        }
    }
    edges.sort();
    edges.dedup();
    edges
}
