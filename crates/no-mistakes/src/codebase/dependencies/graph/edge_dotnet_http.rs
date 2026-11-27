fn prepare_dotnet_http_facts(
    inputs: &GraphEdgeBuildInputs<'_>,
    sources: &SourceStore,
) -> Option<crate::codebase::dotnet::DotnetFactMap> {
    let config = inputs.config_options?;
    if !inputs.plan.http || inputs.dotnet_facts.is_some() || config.dotnet_projects.is_empty() {
        return None;
    }
    Some(crate::codebase::dotnet::collect_dotnet_facts_with_sources(
        inputs.root,
        inputs.graph_files.all(),
        &config.dotnet_projects,
        Some(sources),
    ))
}
