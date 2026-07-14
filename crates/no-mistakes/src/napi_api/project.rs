use std::path::PathBuf;

use super::options::{
    parse_options, parse_queue_direction, parse_server_direction, project_roots,
    resolve_project_root, to_napi_error, DataPwOptions, EffectsOptions, ProjectOptions,
    RegistryExtensionOptions, RscCallersOptions,
};
use crate::cli::root_scoped_edge_depth;

pub(crate) fn queues_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<ProjectOptions>(&options_json)?;
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let tsconfig = options.tsconfig.as_deref().map(PathBuf::from);
    let report = crate::queue::analyze_project(&root, tsconfig.as_deref(), &options.filters)
        .map_err(to_napi_error)?;
    Ok(serde_json::to_string_pretty(&report).expect("queue report serialization never fails"))
}

pub(crate) fn queue_edges_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<ProjectOptions>(&options_json)?;
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let tsconfig = options.tsconfig.as_deref().map(PathBuf::from);
    let report =
        crate::queue::analyze_project_indexed(&root, tsconfig.as_deref(), &options.filters)
            .map_err(to_napi_error)?;
    let depth = root_scoped_edge_depth(&options.files, options.depth);
    let edges = report.edge_view(&options.files, depth);
    Ok(serde_json::to_string_pretty(&edges).expect("queue edge serialization never fails"))
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
    let report =
        crate::queue::analyze_project_indexed(&root, tsconfig.as_deref(), &options.filters)
            .map_err(to_napi_error)?;
    let direction = parse_queue_direction(options.direction.as_deref()).map_err(to_napi_error)?;
    let edges = report.related(&options.files, direction);
    Ok(serde_json::to_string_pretty(&edges).expect("related queue edge serialization never fails"))
}

pub(crate) fn queue_check_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<ProjectOptions>(&options_json)?;
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let tsconfig = options.tsconfig.as_deref().map(PathBuf::from);
    let report = crate::queue::analyze_project(&root, tsconfig.as_deref(), &options.filters)
        .map_err(to_napi_error)?;
    Ok(serde_json::to_string_pretty(&report.check)
        .expect("queue diagnostics serialization never fails"))
}

pub(crate) fn server_routes_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<ProjectOptions>(&options_json)?;
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let tsconfig = options.tsconfig.as_deref().map(PathBuf::from);
    let report =
        crate::server_routes::analyze_project(&root, tsconfig.as_deref(), &options.filters)
            .map_err(to_napi_error)?;
    Ok(serde_json::to_string_pretty(&report).expect("server route serialization never fails"))
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
    Ok(serde_json::to_string_pretty(&routes).expect("server route list serialization never fails"))
}

pub(crate) fn server_route_edges_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<ProjectOptions>(&options_json)?;
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let tsconfig = options.tsconfig.as_deref().map(PathBuf::from);
    let report =
        crate::server_routes::analyze_project_indexed(&root, tsconfig.as_deref(), &options.filters)
            .map_err(to_napi_error)?;
    let roots = project_roots(&options);
    let depth = root_scoped_edge_depth(&roots, options.depth);
    let edges = report.edge_view(&roots, depth);
    Ok(serde_json::to_string_pretty(&edges).expect("server route edge serialization never fails"))
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
        crate::server_routes::analyze_project_indexed(&root, tsconfig.as_deref(), &options.filters)
            .map_err(to_napi_error)?;
    let direction = parse_server_direction(options.direction.as_deref()).map_err(to_napi_error)?;
    let edges = report.related(&roots, direction);
    Ok(serde_json::to_string_pretty(&edges)
        .expect("related server route edge serialization never fails"))
}

include!("project_flow_contracts.rs");

include!("project_query.rs");

pub(crate) fn react_analyze_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<ProjectOptions>(&options_json)?;
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let config = options.config.as_deref().map(PathBuf::from);
    let report =
        crate::react_traits::run_analyze(&root, config.as_deref(), &options.targets, options.depth)
            .map_err(to_napi_error)?;
    Ok(serde_json::to_string_pretty(&report).expect("React analysis serialization never fails"))
}

pub(crate) fn react_usages_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<ProjectOptions>(&options_json)?;
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let config = options.config.as_deref().map(PathBuf::from);
    let target = options.target.ok_or_else(|| {
        napi::Error::from_reason("target is required for react usages".to_string())
    })?;
    let include = crate::react_traits::UsagesInclude::parse(options.include.as_deref())
        .map_err(to_napi_error)?;
    let report = crate::react_traits::run_usages(
        &root,
        config.as_deref(),
        &target,
        &options.targets,
        &include,
    )
    .map_err(to_napi_error)?;
    Ok(serde_json::to_string_pretty(&report).expect("React usage serialization never fails"))
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
    Ok(serde_json::to_string_pretty(&report).expect("React check serialization never fails"))
}
