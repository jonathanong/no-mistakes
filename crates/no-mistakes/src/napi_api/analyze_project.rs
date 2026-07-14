use anyhow::{bail, Result as AnyhowResult};
use serde_json::Value;

use super::codebase::build_traverse_args;
use super::options::{parse_options, to_napi_error};
use crate::codebase::dependencies::TraverseArgs;

mod context;
mod dispatch;
mod options;
mod types;

use dispatch::{graph_direction, is_playwright_report, is_project_report, is_symbols_report};
use options::{flow_options, import_usages_options, symbols_options};
use types::{
    AnalyzeProjectOptions, AnalyzeProjectResult, AnalyzeReportRequest, AnalyzeReportResult,
};

#[cfg(test)]
#[path = "analyze_project/architecture_override_tests.rs"]
mod architecture_override_tests;
#[cfg(test)]
#[path = "analyze_project/cli_parity_tests.rs"]
mod cli_parity_tests;
#[cfg(test)]
#[path = "analyze_project/domain_parity_tests.rs"]
mod domain_parity_tests;
#[cfg(test)]
#[path = "analyze_project/flow_server_tests.rs"]
mod flow_server_tests;
#[cfg(test)]
mod legacy_test_support;
#[cfg(test)]
mod options_test_support;
#[cfg(test)]
mod options_tests;
#[cfg(test)]
mod tests;

pub(crate) fn analyze_project_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<AnalyzeProjectOptions>(&options_json)?;
    let output = analyze_project(options).map_err(to_napi_error)?;
    Ok(serde_json::to_string_pretty(&output)
        .expect("analyzeProject result serialization never fails"))
}

fn analyze_project(options: AnalyzeProjectOptions) -> AnyhowResult<AnalyzeProjectResult> {
    let mut context = context::AnalyzeProjectContext::prepare(&options)?;
    let mut reports = Vec::with_capacity(options.reports.len());

    for request in &options.reports {
        let result = run_report(request, &options, &mut context)?;
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
    context: &mut context::AnalyzeProjectContext,
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
