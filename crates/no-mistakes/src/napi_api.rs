#![cfg_attr(all(coverage, not(test)), allow(dead_code, unused_imports))]

#[cfg(not(coverage))]
use napi::bindgen_prelude::AsyncTask;
#[cfg(all(not(test), not(coverage)))]
use napi_derive::napi;

mod analyze_project;
mod async_task;
mod cli_parity;
mod codebase;
mod infra_swift;
mod lockfile_diff;
mod options;
mod project;

#[cfg(not(coverage))]
use async_task::{JsonTask, VersionTask};
pub(crate) use cli_parity::{
    check_json_impl, fetches_json_impl, playwright_check_json_impl, playwright_edges_json_impl,
    playwright_related_json_impl, playwright_tests_json_impl, tests_comment_markdown_impl,
    tests_graph_json_impl, tests_graph_mermaid_impl, tests_impact_json_impl, tests_plan_json_impl,
    tests_why_json_impl,
};
pub(crate) use codebase::{
    dependencies_json_impl, dependents_json_impl, related_json_impl, symbols_json_impl,
};
#[cfg(not(coverage))]
pub use infra_swift::{
    infra_outputs_json, infra_resource_refs_json, infra_test_for_json, swift_importers_json,
    swift_test_targets_json,
};
pub(crate) use lockfile_diff::lockfile_diff_json_impl;
pub(crate) use project::{
    queue_check_json_impl, queue_edges_json_impl, queue_related_json_impl, queues_json_impl,
    react_analyze_json_impl, react_check_json_impl, react_usages_json_impl,
    server_route_edges_json_impl, server_route_list_json_impl, server_route_related_json_impl,
    server_routes_json_impl,
};

#[cfg(test)]
mod tests;

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi)]
pub fn version() -> AsyncTask<VersionTask> {
    AsyncTask::new(VersionTask)
}

pub(crate) fn version_impl() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "dependenciesJson"))]
pub fn dependencies_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, dependencies_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "dependentsJson"))]
pub fn dependents_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, dependents_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "relatedJson"))]
pub fn related_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, related_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "analyzeProjectJson"))]
pub fn analyze_project_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(
        options_json,
        analyze_project::analyze_project_json_impl,
    ))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "symbolsJson"))]
pub fn symbols_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, symbols_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "fetchesJson"))]
pub fn fetches_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, fetches_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "checkJson"))]
pub fn check_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, check_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "testsPlanJson"))]
pub fn tests_plan_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, tests_plan_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "testsWhyJson"))]
pub fn tests_why_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, tests_why_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "testsCommentMarkdown"))]
pub fn tests_comment_markdown(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, tests_comment_markdown_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "testsGraphJson"))]
pub fn tests_graph_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, tests_graph_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "testsGraphMermaid"))]
pub fn tests_graph_mermaid(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, tests_graph_mermaid_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "testsImpactJson"))]
pub fn tests_impact_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, tests_impact_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "playwrightCheckJson"))]
pub fn playwright_check_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, playwright_check_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "playwrightEdgesJson"))]
pub fn playwright_edges_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, playwright_edges_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "playwrightRelatedJson"))]
pub fn playwright_related_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, playwright_related_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "playwrightTestsJson"))]
pub fn playwright_tests_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, playwright_tests_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "queuesJson"))]
pub fn queues_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, queues_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "queueEdgesJson"))]
pub fn queue_edges_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, queue_edges_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "queueRelatedJson"))]
pub fn queue_related_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, queue_related_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "queueCheckJson"))]
pub fn queue_check_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, queue_check_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "serverRoutesJson"))]
pub fn server_routes_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, server_routes_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "serverRouteListJson"))]
pub fn server_route_list_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, server_route_list_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "serverRouteEdgesJson"))]
pub fn server_route_edges_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, server_route_edges_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "serverRouteRelatedJson"))]
pub fn server_route_related_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, server_route_related_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "reactAnalyzeJson"))]
pub fn react_analyze_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, react_analyze_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "reactCheckJson"))]
pub fn react_check_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, react_check_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "reactUsagesJson"))]
pub fn react_usages_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, react_usages_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "lockfileDiffJson"))]
pub fn lockfile_diff_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, lockfile_diff_json_impl))
}
