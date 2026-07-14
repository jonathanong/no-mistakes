pub(crate) fn server_contracts_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<ProjectOptions>(&options_json)?;
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let tsconfig = options.tsconfig.as_deref().map(PathBuf::from);
    let filters = server_contract_filters(&options);
    let prepared = crate::server_routes::prepare_analysis(&root, tsconfig.as_deref())
        .map_err(to_napi_error)?;
    let report = crate::server_routes::analyze_project_with_prepared(&prepared, &filters)
        .map_err(to_napi_error)?;
    let contracts =
        crate::server_routes::analyze_contracts_with_prepared(&prepared, &report, &filters)
            .map_err(to_napi_error)?;
    Ok(
        serde_json::to_string_pretty(&contracts)
            .expect("server contract serialization never fails"),
    )
}

fn server_contract_filters(options: &ProjectOptions) -> Vec<String> {
    let mut filters = options.filters.clone();
    filters.extend(project_roots(options));
    filters
}

pub(crate) fn flow_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<super::options::FlowOptions>(&options_json)?;
    let options = build_flow_options(options).map_err(to_napi_error)?;
    let report = crate::flow_query::run(&options).map_err(to_napi_error)?;
    Ok(serde_json::to_string_pretty(&report).expect("flow report serialization never fails"))
}

pub(crate) fn build_flow_options(
    options: super::options::FlowOptions,
) -> anyhow::Result<crate::flow_query::FlowOptions> {
    let target = options
        .target
        .ok_or_else(|| anyhow::anyhow!("target is required for flow"))?;
    let root = resolve_project_root(options.root.as_deref())?;
    let relationships = options
        .relationships
        .iter()
        .map(|relationship| super::options::parse_relationship(relationship))
        .collect::<anyhow::Result<Vec<_>>>()?;
    let direction = match options.direction.as_deref().unwrap_or("both") {
        "deps" => crate::flow_query::FlowDirection::Deps,
        "dependents" => crate::flow_query::FlowDirection::Dependents,
        "both" => crate::flow_query::FlowDirection::Both,
        value => anyhow::bail!("unknown flow direction: {value}"),
    };
    Ok(crate::flow_query::FlowOptions {
        target,
        root,
        tsconfig: options.tsconfig.map(PathBuf::from),
        config: options.config.map(PathBuf::from),
        direction,
        depth: options.depth.unwrap_or(2),
        relationships,
    })
}
