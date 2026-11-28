//! `playwright *_json_impl` N-API entry points, split out of [`super`] to
//! stay under the crate's per-file line limit. Re-exported by [`super`] so
//! `cli_parity::playwright_check_json_impl`-style paths (and the bare
//! `napi_api::playwright_check_json_impl` re-export) keep working
//! unchanged.

use super::super::options::{parse_options_value, to_napi_error, PlaywrightOptions};
use std::path::PathBuf;

pub(crate) fn playwright_check_json_impl(options: serde_json::Value) -> napi::Result<String> {
    playwright_json(
        options,
        no_mistakes::playwright::PlaywrightReportKind::Check,
    )
}

pub(crate) fn playwright_edges_json_impl(options: serde_json::Value) -> napi::Result<String> {
    playwright_json(
        options,
        no_mistakes::playwright::PlaywrightReportKind::Edges,
    )
}

pub(crate) fn playwright_related_json_impl(options: serde_json::Value) -> napi::Result<String> {
    playwright_json(
        options,
        no_mistakes::playwright::PlaywrightReportKind::Related,
    )
}

pub(crate) fn playwright_tests_json_impl(options: serde_json::Value) -> napi::Result<String> {
    playwright_json(
        options,
        no_mistakes::playwright::PlaywrightReportKind::Tests,
    )
}

fn playwright_json(
    options: serde_json::Value,
    kind: no_mistakes::playwright::PlaywrightReportKind,
) -> napi::Result<String> {
    let options = parse_options_value::<PlaywrightOptions>(options)?;
    let report_options = no_mistakes::playwright::PlaywrightReportOptions {
        root: options
            .root
            .map(PathBuf::from)
            .unwrap_or(PathBuf::from(".")),
        config: options.config.map(PathBuf::from),
        playwright_config: super::strings_to_paths(options.playwright_config),
        project: options.project,
        app: options.app,
        files: super::strings_to_paths(options.files),
        assert_conditional_tests: options.assert_conditional_tests,
        allow_skipped_tests: options.allow_skipped_tests,
        assert_unique_test_ids: options.assert_unique_test_ids,
        assert_unique_html_ids: options.assert_unique_html_ids,
    };
    no_mistakes::playwright::report_json(kind, report_options).map_err(to_napi_error)
}
