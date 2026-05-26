use std::path::PathBuf;

use anyhow::{bail, Context, Result as AnyhowResult};
use serde_json::json;

use super::options::{
    parse_options, resolve_project_root, to_napi_error, FetchesOptions, PlaywrightOptions,
    ProjectOptions, TestsImpactOptions, TestsPlanDocumentOptions, TestsPlanOptions,
    TestsWhyOptions,
};

include!("cli_parity_builders.rs");

pub(crate) fn fetches_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<FetchesOptions>(&options_json)?;
    let base_root =
        std::env::current_dir().map_err(|error| napi::Error::from_reason(error.to_string()))?;
    let args = crate::fetches::FetchesArgs {
        root: options
            .root
            .map(PathBuf::from)
            .unwrap_or_else(|| ".".into()),
        config: options.config.map(PathBuf::from),
        format: crate::cli::Format::Json,
        json: true,
        targets: options.targets,
    };
    let report =
        crate::fetches::analyze_with_base_root(&base_root, &args).map_err(to_napi_error)?;
    to_pretty_json(&report)
}

pub(crate) fn check_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<ProjectOptions>(&options_json)?;
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let results = crate::check_runner::run_all(
        root,
        options.config.map(PathBuf::from),
        options.tsconfig.map(PathBuf::from),
    )
    .map_err(to_napi_error)?;
    let _has_findings = results.has_findings();
    let crate::check_runner::CheckResults {
        react,
        queues,
        rules,
        integration,
        codebase,
        warnings: _warnings,
        timings: _timings,
    } = results;
    to_pretty_json(&json!({
        "react": react,
        "queues": queues,
        "rules": rules,
        "integration": integration,
        "codebase": codebase,
    }))
}

pub(crate) fn tests_plan_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<TestsPlanOptions>(&options_json)?;
    let args = build_plan_args(options).map_err(to_napi_error)?;
    let plan = crate::tests::plan::generate_plan(&args).map_err(to_napi_error)?;
    to_pretty_json(&plan)
}

pub(crate) fn tests_impact_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<TestsImpactOptions>(&options_json)?;
    let args = build_impact_args(options).map_err(to_napi_error)?;
    let plan = crate::tests::impact::generate_impact_plan(&args).map_err(to_napi_error)?;
    to_pretty_json(&plan)
}

pub(crate) fn tests_why_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<TestsWhyOptions>(&options_json)?;
    let args = build_why_args(options).map_err(to_napi_error)?;
    let steps = crate::tests::why::why_steps(&args).map_err(to_napi_error)?;
    to_pretty_json(&steps)
}

pub(crate) fn tests_comment_markdown_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<TestsPlanDocumentOptions>(&options_json)?;
    let plan = load_plan_document(options).map_err(to_napi_error)?;
    Ok(crate::tests::comment::render_markdown_plan(&plan))
}

pub(crate) fn tests_graph_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<TestsPlanDocumentOptions>(&options_json)?;
    let plan = load_plan_document(options).map_err(to_napi_error)?;
    let graph = crate::tests::graph::graph_json(&plan).map_err(to_napi_error)?;
    to_pretty_json(&graph)
}

pub(crate) fn tests_graph_mermaid_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<TestsPlanDocumentOptions>(&options_json)?;
    let plan = load_plan_document(options).map_err(to_napi_error)?;
    crate::tests::graph::graph_mermaid(&plan).map_err(to_napi_error)
}

pub(crate) fn playwright_check_json_impl(options_json: String) -> napi::Result<String> {
    playwright_json(options_json, crate::playwright::PlaywrightReportKind::Check)
}

pub(crate) fn playwright_edges_json_impl(options_json: String) -> napi::Result<String> {
    playwright_json(options_json, crate::playwright::PlaywrightReportKind::Edges)
}

pub(crate) fn playwright_related_json_impl(options_json: String) -> napi::Result<String> {
    playwright_json(
        options_json,
        crate::playwright::PlaywrightReportKind::Related,
    )
}

pub(crate) fn playwright_tests_json_impl(options_json: String) -> napi::Result<String> {
    playwright_json(options_json, crate::playwright::PlaywrightReportKind::Tests)
}

fn playwright_json(
    options_json: String,
    kind: crate::playwright::PlaywrightReportKind,
) -> napi::Result<String> {
    let options = parse_options::<PlaywrightOptions>(&options_json)?;
    let report_options = crate::playwright::PlaywrightReportOptions {
        root: options
            .root
            .map(PathBuf::from)
            .unwrap_or_else(|| ".".into()),
        config: options.config.map(PathBuf::from),
        playwright_config: strings_to_paths(options.playwright_config),
        project: options.project,
        files: strings_to_paths(options.files),
        assert_conditional_tests: options.assert_conditional_tests,
        allow_skipped_tests: options.allow_skipped_tests,
        assert_unique_test_ids: options.assert_unique_test_ids,
        assert_unique_html_ids: options.assert_unique_html_ids,
        assert_unique_selectors: options.assert_unique_selectors,
    };
    crate::playwright::report_json(kind, report_options).map_err(to_napi_error)
}

fn load_plan_document(options: TestsPlanDocumentOptions) -> AnyhowResult<crate::tests::TestPlan> {
    match (options.plan_json, options.plan) {
        (Some(serde_json::Value::String(raw)), _) => Ok(serde_json::from_str(&raw)?),
        (Some(value), _) => Ok(serde_json::from_value(value)?),
        (None, Some(path)) => {
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read plan from {path}"))?;
            Ok(serde_json::from_str(&content)?)
        }
        (None, None) => bail!("plan or planJson is required"),
    }
}

fn to_pretty_json<T: serde::Serialize>(value: &T) -> napi::Result<String> {
    serde_json::to_string_pretty(value).map_err(|error| napi::Error::from_reason(error.to_string()))
}
