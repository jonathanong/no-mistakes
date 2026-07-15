#![cfg_attr(all(coverage, not(test)), allow(dead_code, unused_imports))]

#[cfg(not(coverage))]
use napi::bindgen_prelude::AsyncTask;
#[cfg(all(not(test), not(coverage)))]
use napi_derive::napi;

mod analyze_project;
#[cfg(feature = "test-instrumentation")]
pub(crate) use analyze_project::analyze_project_json_impl;
mod async_task;
mod cli_parity;
mod codebase;
mod infra_swift;
mod lockfile_diff;
mod options;
mod project;
pub mod queries;

#[cfg(not(coverage))]
use async_task::{JsonTask, VersionTask};
pub(crate) use cli_parity::{
    check_json_impl, ci_env_json_impl, ci_impact_json_impl, fetches_json_impl,
    impacted_checks_json_impl, playwright_check_json_impl, playwright_edges_json_impl,
    playwright_related_json_impl, playwright_tests_json_impl, tests_comment_markdown_impl,
    tests_graph_json_impl, tests_graph_mermaid_impl, tests_impact_json_impl, tests_plan_json_impl,
    tests_targets_json_impl, tests_why_json_impl,
};
pub(crate) use codebase::{
    dependencies_json_impl, dependents_json_impl, import_usages_json_impl, related_json_impl,
    symbols_json_impl,
};
#[cfg(not(coverage))]
pub use infra_swift::{
    infra_outputs_json, infra_resource_refs_json, infra_test_for_json, swift_importers_json,
    swift_test_targets_json,
};
pub(crate) use lockfile_diff::lockfile_diff_json_impl;
pub(crate) use project::{
    data_pw_json_impl, effects_json_impl, flow_json_impl, queue_check_json_impl,
    queue_edges_json_impl, queue_related_json_impl, queues_json_impl, react_analyze_json_impl,
    react_check_json_impl, react_usages_json_impl, registry_extension_json_impl,
    rsc_callers_json_impl, server_contracts_json_impl, server_route_edges_json_impl,
    server_route_list_json_impl, server_route_related_json_impl, server_routes_json_impl,
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

include!("napi_api/codebase_bindings.rs");

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

include!("napi_api/planning_bindings.rs");

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

include!("napi_api/wrappers_query.rs");

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "lockfileDiffJson"))]
pub fn lockfile_diff_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, lockfile_diff_json_impl))
}

include!("napi_api/ci_bindings.rs");
