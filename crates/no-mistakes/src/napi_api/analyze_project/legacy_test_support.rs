use super::options::resolve_root;
use super::types::{AnalyzeProjectOptions, AnalyzeReportRequest};
use crate::codebase::dependencies::{Direction, SharedTraversalContext, TraverseArgs};
use crate::napi_api::codebase::{build_import_usages_args, build_traverse_args};
use anyhow::{bail, Context, Result};
use serde_json::Value;

pub(super) fn graph_report(
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
    direction: Direction,
    shared: Option<&mut SharedTraversalContext>,
) -> Result<Value> {
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

pub(super) fn import_usages_report(
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
    shared: Option<&mut SharedTraversalContext>,
) -> Result<Value> {
    let Some(shared) = shared else {
        bail!("internal error: importUsages report requested without traversal context");
    };
    let value = super::options::import_usages_options(request, options)?;
    let options = serde_json::from_str(value.as_str())?;
    let args = build_import_usages_args(options);
    let cwd = std::env::current_dir().context("reading current directory")?;
    let root = shared.root().to_path_buf();
    let report =
        crate::codebase::import_usages::collect_with_facts(&args, &root, &cwd, shared.facts())?;
    Ok(serde_json::to_value(report)?)
}

pub(super) fn prepare_shared_traversal(
    options: &AnalyzeProjectOptions,
) -> Result<Option<SharedTraversalContext>> {
    if !options.reports.iter().any(|request| {
        matches!(
            request.report_type.as_str(),
            "dependencies" | "dependents" | "related" | "importUsages"
        )
    }) {
        return Ok(None);
    }

    let root = resolve_root(options.root.as_deref())?;
    let mut build_plan = crate::codebase::dependencies::graph::GraphBuildPlan::default();
    let mut requested_frameworks = Vec::new();
    for request in &options.reports {
        if matches!(
            request.report_type.as_str(),
            "dependencies" | "dependents" | "related"
        ) {
            let args = traverse_args(request, options)?;
            requested_frameworks.extend(args.tests.iter().cloned());
            let allowed = crate::codebase::dependencies::relationship_filter(&args.relationships);
            build_plan.include(
                crate::codebase::dependencies::graph::GraphBuildPlan::from_allowed(
                    allowed.as_ref(),
                )
                .with_symbols(args.include_symbols),
            );
        } else if request.report_type == "importUsages" {
            build_plan.include(crate::codebase::dependencies::graph::GraphBuildPlan {
                imports: true,
                ..Default::default()
            });
        }
    }

    let mut framework_plan =
        crate::codebase::test_discovery::FrameworkPreparationPlan::for_graph(build_plan);
    framework_plan.include_framework_names(requested_frameworks.iter().map(String::as_str));
    Ok(Some(SharedTraversalContext::prepare_with_framework_plan(
        root,
        options.tsconfig.as_deref().map(std::path::Path::new),
        options.config.as_deref().map(std::path::Path::new),
        build_plan,
        framework_plan,
    )?))
}

fn traverse_args(
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
) -> Result<TraverseArgs> {
    build_traverse_args(super::options::traverse_options(request, options)?)
}
