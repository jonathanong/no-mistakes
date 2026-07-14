use std::path::{Path, PathBuf};

use super::options::{
    parse_options, resolve_project_root, to_napi_error, CiEnvOptions, CiImpactOptions,
    FetchesOptions, ImpactedChecksOptions, PlaywrightOptions, ProjectOptions, TestsImpactOptions,
    TestsPlanDocumentOptions, TestsPlanOptions, TestsTargetsOptions, TestsWhyOptions,
};
use anyhow::{bail, Context, Result as AnyhowResult};

include!("cli_parity_builders.rs");

pub(crate) fn fetches_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<FetchesOptions>(&options_json)?;
    let base_root = std::env::current_dir()
        .map_err(anyhow::Error::from)
        .map_err(to_napi_error)?;
    let args = crate::fetches::FetchesArgs {
        root: options
            .root
            .map(PathBuf::from)
            .unwrap_or(PathBuf::from(".")),
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
    to_pretty_json(&crate::check_runner::json_value(&results))
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

pub(crate) fn tests_targets_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<TestsTargetsOptions>(&options_json)?;
    let framework = options
        .framework
        .as_deref()
        .map(parse_test_framework)
        .transpose()
        .map_err(to_napi_error)?
        .context("framework is required")
        .map_err(to_napi_error)?;
    if options.files.is_empty() {
        return Err(to_napi_error(anyhow::anyhow!("files is required")));
    }
    let args = crate::tests::TargetsArgs {
        framework,
        files: options.files.into_iter().map(PathBuf::from).collect(),
        root: options
            .root
            .map(PathBuf::from)
            .unwrap_or(PathBuf::from(".")),
        config: options.config.map(PathBuf::from),
        format: None,
        json: true,
    };
    let report = crate::tests::targets::generate_targets(&args).map_err(to_napi_error)?;
    to_pretty_json(&report)
}

pub(crate) fn ci_impact_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<CiImpactOptions>(&options_json)?;
    let root = options.root.unwrap_or(String::from("."));
    let files: Vec<PathBuf> = options.files.into_iter().map(PathBuf::from).collect();
    let report = crate::ci::impact_report(
        Path::new(&root),
        options.config.as_deref().map(Path::new),
        &files,
    )
    .map_err(to_napi_error)?;
    to_pretty_json(&report)
}

pub(crate) fn ci_env_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<CiEnvOptions>(&options_json)?;
    let var = options
        .var
        .context("var is required")
        .map_err(to_napi_error)?;
    let root = options.root.unwrap_or(String::from("."));
    let report = crate::ci::env_report(
        Path::new(&root),
        options.config.as_deref().map(Path::new),
        &var,
    )
    .map_err(to_napi_error)?;
    to_pretty_json(&report)
}

pub(crate) fn impacted_checks_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<ImpactedChecksOptions>(&options_json)?;
    let collect_timings = options.timings;
    let args = build_impacted_checks_args(options);
    let mut timing = crate::impacted_checks::timing::TimingTracker::new(false, collect_timings);
    let (report, _) =
        crate::impacted_checks::generate_impacted_checks_with_timing(&args, &mut timing)
            .map_err(to_napi_error)?;
    timing.finish_total();
    let Some(timings) = timing.into_timings() else {
        return to_pretty_json(&report);
    };
    let mut value = serde_json::to_value(&report).map_err(|error| to_napi_error(error.into()))?;
    value["timings"] =
        serde_json::to_value(timings).map_err(|error| to_napi_error(error.into()))?;
    to_pretty_json(&value)
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
            .unwrap_or(PathBuf::from(".")),
        config: options.config.map(PathBuf::from),
        playwright_config: strings_to_paths(options.playwright_config),
        project: options.project,
        files: strings_to_paths(options.files),
        assert_conditional_tests: options.assert_conditional_tests,
        allow_skipped_tests: options.allow_skipped_tests,
        assert_unique_test_ids: options.assert_unique_test_ids,
        assert_unique_html_ids: options.assert_unique_html_ids,
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
    Ok(serde_json::to_string_pretty(value).expect("N-API report serialization never fails"))
}
