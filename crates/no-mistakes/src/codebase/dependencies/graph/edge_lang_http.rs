fn collect_lang_http_client_edges(
    inputs: &GraphEdgeBuildInputs<'_>,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> Vec<Edge> {
    let Some(options) = inputs.config_options else {
        return Vec::new();
    };
    let config = lang_config_from_options(options);
    let facts = (!config_is_empty(&config)).then(|| {
        let files = inputs.graph_files.all_shared();
        let sources = graph_edge_sources(session, inputs);
        session.lang_frontend_facts(inputs.root, &files, &config, &sources)
    });
    let mut files = Vec::new();
    if let Some(facts) = &facts {
        files.extend(
            crate::codebase::lang_frontends::each_lang_map(facts)
                .into_iter()
                .flat_map(|map| map.files.values())
                .map(|file| HttpClientFile {
                    path: file.path.as_path(),
                    calls: &file.http_calls,
                    routes: &file.route_handlers,
                }),
        );
    }
    extend_dotnet_http_client_files(inputs.dotnet_facts, &mut files);
    http_edges_for_client_files(&files, &inputs.interner)
}

fn extend_dotnet_http_client_files<'a>(
    facts: Option<&'a crate::codebase::dotnet::DotnetFactMap>,
    files: &mut Vec<HttpClientFile<'a>>,
) {
    files.extend(
        facts
            .into_iter()
            .flat_map(|facts| facts.files.values())
            .map(|file| HttpClientFile {
                path: file.path.as_path(),
                calls: &file.http_calls,
                routes: &file.route_handlers,
            }),
    );
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
