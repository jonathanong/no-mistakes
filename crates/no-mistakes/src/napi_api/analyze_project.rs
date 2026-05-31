use anyhow::{bail, Context, Result as AnyhowResult};
use serde_json::Value;

use super::codebase::build_traverse_args;
use super::options::{parse_options, to_napi_error};
use crate::codebase::dependencies::{Direction, SharedTraversalContext, TraverseArgs};

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
        let result = match request.report_type.as_str() {
            "dependencies" => graph_report(request, &options, Direction::Deps, shared.as_mut())?,
            "dependents" | "related" => {
                graph_report(request, &options, Direction::Dependents, shared.as_mut())?
            }
            "symbols" => call_report(
                symbols_options(request, &options)?,
                super::symbols_json_impl,
            )?,
            "queues" => call_report(project_options(request, &options)?, super::queues_json_impl)?,
            "queueEdges" => call_report(
                project_options(request, &options)?,
                super::queue_edges_json_impl,
            )?,
            "queueRelated" => call_report(
                project_options(request, &options)?,
                super::queue_related_json_impl,
            )?,
            "queueCheck" => call_report(
                project_options(request, &options)?,
                super::queue_check_json_impl,
            )?,
            "serverRoutes" => call_report(
                project_options(request, &options)?,
                super::server_routes_json_impl,
            )?,
            "serverRouteList" => call_report(
                project_options(request, &options)?,
                super::server_route_list_json_impl,
            )?,
            "serverRouteEdges" => call_report(
                project_options(request, &options)?,
                super::server_route_edges_json_impl,
            )?,
            "serverRouteRelated" => call_report(
                project_options(request, &options)?,
                super::server_route_related_json_impl,
            )?,
            "reactAnalyze" => call_report(
                project_options(request, &options)?,
                super::react_analyze_json_impl,
            )?,
            "reactCheck" => call_report(
                project_options(request, &options)?,
                super::react_check_json_impl,
            )?,
            "playwrightCheck" => call_report(
                playwright_options(request, &options)?,
                super::playwright_check_json_impl,
            )?,
            "playwrightEdges" => call_report(
                playwright_options(request, &options)?,
                super::playwright_edges_json_impl,
            )?,
            "playwrightRelated" => call_report(
                playwright_options(request, &options)?,
                super::playwright_related_json_impl,
            )?,
            "playwrightTests" => call_report(
                playwright_options(request, &options)?,
                super::playwright_tests_json_impl,
            )?,
            "check" => call_report(project_options(request, &options)?, super::check_json_impl)?,
            value => bail!("unknown analyzeProject report type: {value}"),
        };
        reports.push(AnalyzeReportResult {
            id: request.id.clone(),
            report_type: request.report_type.clone(),
            result,
        });
    }

    Ok(AnalyzeProjectResult { reports })
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
                ),
            );
        }
    }

    Ok(Some(shared))
}

fn traverse_args(
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
) -> AnyhowResult<TraverseArgs> {
    build_traverse_args(options::traverse_options(request, options)?)
}

fn call_report(
    options_json: String,
    run: fn(String) -> napi::Result<String>,
) -> AnyhowResult<Value> {
    let raw = run(options_json).map_err(|error| anyhow::anyhow!("{}", error.reason))?;
    Ok(serde_json::from_str(&raw)?)
}
