use anyhow::{bail, Result as AnyhowResult};
use rayon::prelude::*;
use serde_json::Value;

use super::codebase::build_traverse_args;
#[cfg(any(test, feature = "test-instrumentation"))]
use super::options::parse_options;
use super::options::to_napi_error;
use crate::codebase::dependencies::TraverseArgs;

mod context;
mod dispatch;
mod options;
mod types;

use dispatch::{
    graph_direction, is_command_report, is_playwright_report, is_project_report, is_symbols_report,
};
use options::{flow_options, import_usages_options, symbols_options};
use types::{
    AnalyzeProjectOptions, AnalyzeProjectResult, AnalyzeReportRequest, AnalyzeReportResult,
};

#[cfg(test)]
#[path = "analyze_project/tests/architecture_override.rs"]
mod architecture_override_tests;
#[cfg(test)]
#[path = "analyze_project/cli_parity_tests.rs"]
mod cli_parity_tests;
#[cfg(test)]
#[path = "analyze_project/command_report_tests.rs"]
mod command_report_tests;
#[cfg(test)]
#[path = "analyze_project/domain_parity_tests.rs"]
mod domain_parity_tests;
#[cfg(test)]
#[path = "analyze_project/flow_server_tests.rs"]
mod flow_server_tests;
#[cfg(test)]
#[path = "analyze_project/import_usages_scope_tests.rs"]
mod import_usages_scope_tests;
#[cfg(test)]
mod legacy_test_support;
#[cfg(test)]
mod options_test_support;
#[cfg(test)]
mod options_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_dispatch;
#[cfg(test)]
#[path = "analyze_project/tracked_banned_paths_tests.rs"]
mod tracked_banned_paths_tests;

#[cfg(any(test, feature = "test-instrumentation"))]
pub(crate) fn analyze_project_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<AnalyzeProjectOptions>(&options_json)?;
    analyze_project_options_impl(options)
}

pub(crate) fn analyze_project_value_impl(options: Value) -> napi::Result<String> {
    let options = serde_json::from_value::<AnalyzeProjectOptions>(options)
        .map_err(|error| napi::Error::from_reason(format!("invalid options JSON: {error}")))?;
    analyze_project_options_impl(options)
}

fn analyze_project_options_impl(options: AnalyzeProjectOptions) -> napi::Result<String> {
    let output = analyze_project(options).map_err(to_napi_error)?;
    Ok(serde_json::to_string(&output).expect("analyzeProject result serialization never fails"))
}

fn analyze_project(options: AnalyzeProjectOptions) -> AnyhowResult<AnalyzeProjectResult> {
    let context = context::AnalyzeProjectContext::prepare(&options)?;
    let observer = crate::diagnostics::current();
    let reports = options
        .reports
        .par_iter()
        .map(|request| {
            crate::diagnostics::with_observer(observer.clone(), || {
                run_report(request, &options, &context).map(|result| AnalyzeReportResult {
                    id: request.id.clone(),
                    report_type: request.report_type.clone(),
                    result,
                })
            })
        })
        .collect::<Vec<_>>();
    let reports = reports.into_iter().collect::<AnyhowResult<Vec<_>>>()?;

    Ok(AnalyzeProjectResult { reports })
}

fn run_report(
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
    context: &context::AnalyzeProjectContext,
) -> AnyhowResult<Value> {
    if let Some(direction) = graph_direction(&request.report_type) {
        return context.graph_report(request, options, direction);
    }
    if is_symbols_report(&request.report_type) {
        return context.symbols_report(request, options);
    }
    if request.report_type == "importUsages" {
        return context.import_usages_report(request, options);
    }
    if is_playwright_report(&request.report_type) {
        return context.playwright_report(request, options);
    }
    if request.report_type == "flow" {
        return context.flow_report(request, options);
    }
    if request.report_type == "effects" {
        return context.effects_report(request, options);
    }
    if request.report_type == "rscCallers" {
        return context.rsc_callers_report(request, options);
    }
    if is_project_report(&request.report_type) {
        return context.project_report(request, options);
    }
    if is_command_report(&request.report_type) {
        return context.command_report(request, options);
    }
    bail!(
        "unknown analyzeProject report type: {}",
        request.report_type
    )
}

fn is_server_report(report_type: &str) -> bool {
    matches!(
        report_type,
        "serverRoutes"
            | "serverRouteList"
            | "serverRouteEdges"
            | "serverRouteRelated"
            | "serverContracts"
    )
}

fn traverse_args(
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
) -> AnyhowResult<TraverseArgs> {
    build_traverse_args(options::traverse_options(request, options)?)
}
