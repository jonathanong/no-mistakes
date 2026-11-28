use super::options::to_napi_error;
use super::project::project_setup;
use anyhow::bail;
use no_mistakes::cli::root_scoped_edge_depth;
use no_mistakes::queue::{
    analyze_project, analyze_project_indexed, analyze_project_with_config, CheckReport,
    RelatedDirection,
};

pub(crate) fn parse_queue_direction(value: Option<&str>) -> anyhow::Result<RelatedDirection> {
    match value.unwrap_or("both") {
        "deps" => Ok(RelatedDirection::Deps),
        "dependents" => Ok(RelatedDirection::Dependents),
        "both" => Ok(RelatedDirection::Both),
        value => bail!("unknown direction: {value}"),
    }
}

pub(crate) fn queues_json_impl(options: serde_json::Value) -> napi::Result<String> {
    let (options, root, tsconfig) = project_setup(options)?;
    let report =
        analyze_project(&root, tsconfig.as_deref(), &options.filters).map_err(to_napi_error)?;
    Ok(no_mistakes::cli::json_string(&report))
}

pub(crate) fn queue_edges_json_impl(options: serde_json::Value) -> napi::Result<String> {
    let (options, root, tsconfig) = project_setup(options)?;
    let report = analyze_project_indexed(&root, tsconfig.as_deref(), &options.filters)
        .map_err(to_napi_error)?;
    let depth = root_scoped_edge_depth(&options.files, options.depth);
    let edges = report.edge_view(&options.files, depth);
    Ok(no_mistakes::cli::json_string(&edges))
}

pub(crate) fn queue_related_json_impl(options: serde_json::Value) -> napi::Result<String> {
    let (options, root, tsconfig) = project_setup(options)?;
    if options.files.is_empty() {
        return Err(napi::Error::from_reason(
            "files must contain at least one file".to_string(),
        ));
    }
    let report = analyze_project_indexed(&root, tsconfig.as_deref(), &options.filters)
        .map_err(to_napi_error)?;
    let direction = parse_queue_direction(options.direction.as_deref()).map_err(to_napi_error)?;
    let edges = report.related(&options.files, direction);
    Ok(no_mistakes::cli::json_string(&edges))
}

pub(crate) fn queue_check_json_impl(options: serde_json::Value) -> napi::Result<String> {
    let (options, root, tsconfig) = project_setup(options)?;
    let config = options.config.as_deref().map(std::path::Path::new);
    let report = analyze_project_with_config(&root, tsconfig.as_deref(), config, &options.filters)
        .map_err(to_napi_error)?;
    if options.include_diagnostics {
        Ok(no_mistakes::cli::json_string(&CheckReport {
            findings: &report.check,
            diagnostics: &report.diagnostics,
        }))
    } else {
        Ok(no_mistakes::cli::json_string(&report.check))
    }
}
