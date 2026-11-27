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
    use crate::codebase::lang_frontends::normalize_route_pattern;
    use crate::codebase::ts_routes::matcher;

    let mut callers = files.iter().filter(|file| !file.calls.is_empty()).peekable();
    if callers.peek().is_none() {
        return Vec::new();
    }
    let routes: Vec<_> = files
        .iter()
        .flat_map(|file| {
            file.routes
                .iter()
                .map(|(route, _)| (file.path, normalize_route_pattern(route)))
        })
        .collect();
    let mut edges = Vec::new();
    for caller in callers {
        for call in caller.calls {
            for (route_file, pattern) in &routes {
                if *route_file != caller.path && matcher::matches(call, pattern) {
                    edges.push((
                        NodeId::file_in(interner, caller.path),
                        NodeId::file_in(interner, route_file),
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
