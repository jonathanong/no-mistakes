use std::path::PathBuf;

use super::options::{
    parse_options, parse_queue_direction, parse_server_direction, project_roots,
    resolve_project_root, to_napi_error, ProjectOptions,
};
use crate::cli::{edge_view, root_scoped_edge_depth};

pub(crate) fn queues_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<ProjectOptions>(&options_json)?;
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let tsconfig = options.tsconfig.as_deref().map(PathBuf::from);
    let report = crate::queue::analyze_project(&root, tsconfig.as_deref(), &options.filters)
        .map_err(to_napi_error)?;
    serde_json::to_string_pretty(&report)
        .map_err(|error| napi::Error::from_reason(error.to_string()))
}

pub(crate) fn queue_edges_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<ProjectOptions>(&options_json)?;
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let tsconfig = options.tsconfig.as_deref().map(PathBuf::from);
    let report = crate::queue::analyze_project(&root, tsconfig.as_deref(), &options.filters)
        .map_err(to_napi_error)?;
    let depth = root_scoped_edge_depth(&options.files, options.depth);
    let edges = edge_view(&report.edges, &options.files, depth);
    serde_json::to_string_pretty(&edges)
        .map_err(|error| napi::Error::from_reason(error.to_string()))
}

pub(crate) fn queue_related_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<ProjectOptions>(&options_json)?;
    if options.files.is_empty() {
        return Err(napi::Error::from_reason(
            "files must contain at least one file".to_string(),
        ));
    }
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let tsconfig = options.tsconfig.as_deref().map(PathBuf::from);
    let report = crate::queue::analyze_project(&root, tsconfig.as_deref(), &options.filters)
        .map_err(to_napi_error)?;
    let direction = parse_queue_direction(options.direction.as_deref()).map_err(to_napi_error)?;
    let edges = crate::queue::related(&report, &options.files, direction);
    serde_json::to_string_pretty(&edges)
        .map_err(|error| napi::Error::from_reason(error.to_string()))
}

pub(crate) fn queue_check_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<ProjectOptions>(&options_json)?;
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let tsconfig = options.tsconfig.as_deref().map(PathBuf::from);
    let report = crate::queue::analyze_project(&root, tsconfig.as_deref(), &options.filters)
        .map_err(to_napi_error)?;
    serde_json::to_string_pretty(&report.check)
        .map_err(|error| napi::Error::from_reason(error.to_string()))
}

pub(crate) fn server_routes_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<ProjectOptions>(&options_json)?;
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let tsconfig = options.tsconfig.as_deref().map(PathBuf::from);
    let report =
        crate::server_routes::analyze_project(&root, tsconfig.as_deref(), &options.filters)
            .map_err(to_napi_error)?;
    serde_json::to_string_pretty(&report)
        .map_err(|error| napi::Error::from_reason(error.to_string()))
}

pub(crate) fn server_route_list_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<ProjectOptions>(&options_json)?;
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let tsconfig = options.tsconfig.as_deref().map(PathBuf::from);
    let report =
        crate::server_routes::analyze_project(&root, tsconfig.as_deref(), &options.filters)
            .map_err(to_napi_error)?;
    let routes: Vec<&crate::server_routes::ServerRoute> = if options.files.is_empty() {
        report.routes.iter().collect()
    } else {
        report
            .routes
            .iter()
            .filter(|route| {
                options
                    .files
                    .iter()
                    .any(|file| file == &route.file || file == &route.route)
            })
            .collect()
    };
    serde_json::to_string_pretty(&routes)
        .map_err(|error| napi::Error::from_reason(error.to_string()))
}

pub(crate) fn server_route_edges_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<ProjectOptions>(&options_json)?;
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let tsconfig = options.tsconfig.as_deref().map(PathBuf::from);
    let report =
        crate::server_routes::analyze_project(&root, tsconfig.as_deref(), &options.filters)
            .map_err(to_napi_error)?;
    let roots = project_roots(&options);
    let depth = root_scoped_edge_depth(&roots, options.depth);
    let edges = edge_view(&report.edges, &roots, depth);
    serde_json::to_string_pretty(&edges)
        .map_err(|error| napi::Error::from_reason(error.to_string()))
}

pub(crate) fn server_route_related_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<ProjectOptions>(&options_json)?;
    let roots = project_roots(&options);
    if roots.is_empty() {
        return Err(napi::Error::from_reason(
            "files or roots must contain at least one entry".to_string(),
        ));
    }
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let tsconfig = options.tsconfig.as_deref().map(PathBuf::from);
    let report =
        crate::server_routes::analyze_project(&root, tsconfig.as_deref(), &options.filters)
            .map_err(to_napi_error)?;
    let direction = parse_server_direction(options.direction.as_deref()).map_err(to_napi_error)?;
    let edges = crate::server_routes::related(&report, &roots, direction);
    serde_json::to_string_pretty(&edges)
        .map_err(|error| napi::Error::from_reason(error.to_string()))
}

pub(crate) fn react_analyze_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<ProjectOptions>(&options_json)?;
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let config = options.config.as_deref().map(PathBuf::from);
    let report =
        crate::react_traits::run_analyze(&root, config.as_deref(), &options.targets, options.depth)
            .map_err(to_napi_error)?;
    serde_json::to_string_pretty(&report)
        .map_err(|error| napi::Error::from_reason(error.to_string()))
}

pub(crate) fn react_check_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<ProjectOptions>(&options_json)?;
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let config = options.config.as_deref().map(PathBuf::from);
    let report = crate::react_traits::run_check(
        &root,
        config.as_deref(),
        &options.targets,
        options.assert_no_fetch,
    )
    .map_err(to_napi_error)?;
    serde_json::to_string_pretty(&report)
        .map_err(|error| napi::Error::from_reason(error.to_string()))
}
