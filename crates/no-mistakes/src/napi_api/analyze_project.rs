use anyhow::{bail, Context, Result as AnyhowResult};
use serde_json::Value;

use super::codebase::build_traverse_args;
use super::options::{parse_options, to_napi_error};
use crate::codebase::dependencies::{Direction, SharedTraversalContext, TraverseArgs};

type ReportRunner = fn(String) -> napi::Result<String>;

mod options;
mod types;

use options::{
    playwright_options, project_options, resolve_root, resolve_tsconfig, symbols_options,
};
use types::{
    AnalyzeProjectOptions, AnalyzeProjectResult, AnalyzeReportRequest, AnalyzeReportResult,
};

#[cfg(test)]
mod options_tests;
#[cfg(test)]
mod tests;

pub(crate) fn analyze_project_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<AnalyzeProjectOptions>(&options_json)?;
    let output = analyze_project(options).map_err(to_napi_error)?;
    serde_json::to_string_pretty(&output)
        .map_err(|error| napi::Error::from_reason(error.to_string()))
}

fn analyze_project(options: AnalyzeProjectOptions) -> AnyhowResult<AnalyzeProjectResult> {
    let mut shared = prepare_shared_traversal(&options)?;
    let mut reports = Vec::with_capacity(options.reports.len());

    for request in &options.reports {
        let result = run_report(request, &options, shared.as_mut())?;
        reports.push(AnalyzeReportResult {
            id: request.id.clone(),
            report_type: request.report_type.clone(),
            result,
        });
    }

    Ok(AnalyzeProjectResult { reports })
}

fn run_report(
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
    shared: Option<&mut SharedTraversalContext>,
) -> AnyhowResult<Value> {
    if let Some(direction) = graph_direction(&request.report_type) {
        return graph_report(request, options, direction, shared);
    }
    if let Some(run) = symbols_runner(&request.report_type) {
        return call_report(symbols_options(request, options)?, run);
    }
    if let Some(run) = playwright_runner(&request.report_type) {
        return call_report(playwright_options(request, options)?, run);
    }
    if let Some(run) = project_runner(&request.report_type) {
        return call_report(project_options(request, options)?, run);
    }
    bail!(
        "unknown analyzeProject report type: {}",
        request.report_type
    )
}

fn graph_direction(report_type: &str) -> Option<Direction> {
    match report_type {
        "dependencies" => Some(Direction::Deps),
        "dependents" | "related" => Some(Direction::Dependents),
        _ => None,
    }
}

fn symbols_runner(report_type: &str) -> Option<ReportRunner> {
    match report_type {
        "symbols" => Some(super::symbols_json_impl),
        _ => None,
    }
}

fn playwright_runner(report_type: &str) -> Option<ReportRunner> {
    match report_type {
        "playwrightCheck" => Some(super::playwright_check_json_impl),
        "playwrightEdges" => Some(super::playwright_edges_json_impl),
        "playwrightRelated" => Some(super::playwright_related_json_impl),
        "playwrightTests" => Some(super::playwright_tests_json_impl),
        _ => None,
    }
}

fn project_runner(report_type: &str) -> Option<ReportRunner> {
    match report_type {
        "queues" => Some(super::queues_json_impl),
        "queueEdges" => Some(super::queue_edges_json_impl),
        "queueRelated" => Some(super::queue_related_json_impl),
        "queueCheck" => Some(super::queue_check_json_impl),
        "serverRoutes" => Some(super::server_routes_json_impl),
        "serverRouteList" => Some(super::server_route_list_json_impl),
        "serverRouteEdges" => Some(super::server_route_edges_json_impl),
        "serverRouteRelated" => Some(super::server_route_related_json_impl),
        "reactAnalyze" => Some(super::react_analyze_json_impl),
        "reactCheck" => Some(super::react_check_json_impl),
        "reactUsages" => Some(super::react_usages_json_impl),
        "check" => Some(super::check_json_impl),
        _ => None,
    }
}

fn graph_report(
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
    direction: Direction,
    shared: Option<&mut SharedTraversalContext>,
) -> AnyhowResult<Value> {
    let Some(shared) = shared else {
        bail!("internal error: graph report requested without traversal context");
    };
    let args = traverse_args(request, options)?;
    let cwd = std::env::current_dir().context("reading current directory")?;
    let result = crate::codebase::dependencies::collect_and_filter_entries_shared(
        &args, direction, &cwd, shared,
    )?;
    let json = crate::codebase::dependencies::result_json(&args, &result)?;
    Ok(serde_json::from_str(&json)?)
}

fn prepare_shared_traversal(
    options: &AnalyzeProjectOptions,
) -> AnyhowResult<Option<SharedTraversalContext>> {
    if !options.reports.iter().any(|request| {
        matches!(
            request.report_type.as_str(),
            "dependencies" | "dependents" | "related"
        )
    }) {
        return Ok(None);
    }

    let root = resolve_root(options.root.as_deref())?;
    let tsconfig = resolve_tsconfig(&root, options.tsconfig.as_deref())?;
    let graph_files = crate::codebase::dependencies::graph::GraphFiles::discover(&root);
    let mut shared = SharedTraversalContext::new(root, tsconfig, graph_files);

    for request in &options.reports {
        if matches!(
            request.report_type.as_str(),
            "dependencies" | "dependents" | "related"
        ) {
            let args = traverse_args(request, options)?;
            let allowed = crate::codebase::dependencies::relationship_filter(&args.relationships);
            shared.include_plan(
                crate::codebase::dependencies::graph::GraphBuildPlan::from_allowed(
                    allowed.as_ref(),
                )
                .with_symbols(args.include_symbols),
            );
        }
    }

    Ok(Some(shared))
}

fn traverse_args(
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
) -> AnyhowResult<TraverseArgs> {
    reject_graph_scope_overrides(request)?;
    build_traverse_args(options::traverse_options(request, options)?)
}

fn reject_graph_scope_overrides(request: &AnalyzeReportRequest) -> AnyhowResult<()> {
    if request.options.contains_key("root") || request.options.contains_key("tsconfig") {
        bail!(
            "graph reports in analyzeProject must use top-level root and tsconfig; per-report root/tsconfig overrides are not supported"
        );
    }
    Ok(())
}

fn call_report(
    options_json: String,
    run: fn(String) -> napi::Result<String>,
) -> AnyhowResult<Value> {
    let raw = run(options_json).map_err(|error| anyhow::anyhow!("{}", error.reason))?;
    Ok(serde_json::from_str(&raw)?)
}
