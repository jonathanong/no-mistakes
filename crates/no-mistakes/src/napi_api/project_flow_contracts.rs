pub(crate) fn server_contracts_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<ProjectOptions>(&options_json)?;
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let tsconfig = options.tsconfig.as_deref().map(PathBuf::from);
    let report =
        crate::server_routes::analyze_project(&root, tsconfig.as_deref(), &options.filters)
            .map_err(to_napi_error)?;
    let contracts = crate::server_routes::analyze_contracts(&root, &report);
    serde_json::to_string_pretty(&contracts)
        .map_err(|error| napi::Error::from_reason(error.to_string()))
}

pub(crate) fn flow_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<super::options::FlowOptions>(&options_json)?;
    let target = options
        .target
        .ok_or_else(|| napi::Error::from_reason("target is required for flow".to_string()))?;
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let relationships = options
        .relationships
        .iter()
        .map(|relationship| super::options::parse_relationship(relationship))
        .collect::<anyhow::Result<Vec<_>>>()
        .map_err(to_napi_error)?;
    let direction = match options.direction.as_deref().unwrap_or("both") {
        "deps" => crate::flow_query::FlowDirection::Deps,
        "dependents" => crate::flow_query::FlowDirection::Dependents,
        "both" => crate::flow_query::FlowDirection::Both,
        value => {
            return Err(napi::Error::from_reason(format!(
                "unknown flow direction: {value}"
            )));
        }
    };
    let report = crate::flow_query::run(&crate::flow_query::FlowOptions {
        target,
        root,
        tsconfig: options.tsconfig.map(PathBuf::from),
        config: options.config.map(PathBuf::from),
        direction,
        depth: options.depth.unwrap_or(2),
        relationships,
    })
    .map_err(to_napi_error)?;
    serde_json::to_string_pretty(&report)
        .map_err(|error| napi::Error::from_reason(error.to_string()))
}
