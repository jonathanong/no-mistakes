fn dotnet_http_client_edges(
    facts: &crate::codebase::dotnet::DotnetFactMap,
    interner: &PathInterner,
) -> Vec<Edge> {
    let files: Vec<HttpClientFile<'_>> = facts
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

fn dotnet_http_edges_for_plan(
    inputs: &GraphEdgeBuildInputs<'_>,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> Vec<Edge> {
    if inputs.plan.dotnet {
        return Vec::new();
    }
    let Some(config_options) = inputs.config_options else {
        return Vec::new();
    };
    if config_options.dotnet_projects.is_empty() {
        return Vec::new();
    }
    let sources = graph_edge_sources(session, inputs);
    let owned = inputs.dotnet_facts.is_none().then(|| {
        crate::codebase::dotnet::collect_dotnet_facts_with_sources(
            inputs.root,
            inputs.graph_files.all(),
            &config_options.dotnet_projects,
            Some(sources.as_ref()),
        )
    });
    let Some(facts) = inputs.dotnet_facts.or(owned.as_ref()) else {
        return Vec::new();
    };
    dotnet_http_client_edges(facts, &inputs.interner)
}
