#![cfg_attr(all(coverage, not(test)), allow(dead_code, unused_imports))]

#[cfg(not(coverage))]
use napi::bindgen_prelude::AsyncTask;
#[cfg(all(not(test), not(coverage)))]
use napi_derive::napi;

// Keep every JSON N-API entrypoint on libuv while making its Rust and
// JavaScript names explicit at the registration site. Defined before the
// child modules so their registrations use the same declarative boundary.
macro_rules! json_binding {
    ($rust_name:ident, $js_name:literal, $implementation:path) => {
        json_binding!($rust_name, $js_name, $implementation, value);
    };
    ($rust_name:ident, $js_name:literal, $implementation:path, value) => {
        #[cfg(not(coverage))]
        #[cfg_attr(not(test), napi(js_name = $js_name))]
        pub fn $rust_name(options_json: napi::bindgen_prelude::Buffer) -> AsyncTask<JsonValueTask> {
            AsyncTask::new(JsonValueTask::new(options_json, $implementation))
        }
    };
    ($rust_name:ident, $js_name:literal, $implementation:path, string) => {
        #[cfg(not(coverage))]
        #[cfg_attr(not(test), napi(js_name = $js_name))]
        pub fn $rust_name(options_json: napi::bindgen_prelude::Buffer) -> AsyncTask<JsonTask> {
            AsyncTask::new(JsonTask::new(options_json, $implementation))
        }
    };
}

mod analyze_project;
#[cfg(feature = "test-instrumentation")]
pub(crate) use analyze_project::analyze_project_json_impl;
mod async_task;
mod cli_parity;
mod codebase;
pub(crate) mod infra_swift;
mod lockfile_diff;
#[cfg(feature = "mermaid-validation")]
mod mermaid;
pub(crate) mod options;
mod project;
pub mod queries;

#[cfg(not(coverage))]
#[allow(unused_imports)]
use async_task::{JsonTask, JsonValueTask, VersionTask};
pub(crate) use cli_parity::{
    check_json_impl, ci_env_json_impl, ci_impact_json_impl, ci_topology_json_impl,
    fetches_json_impl, impacted_checks_json_impl, playwright_check_json_impl,
    playwright_edges_json_impl, playwright_related_json_impl, playwright_tests_json_impl,
    resolve_config_json_impl, tests_comment_markdown_impl, tests_graph_json_impl,
    tests_graph_mermaid_impl, tests_impact_json_impl, tests_plan_json_impl,
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
// json_binding! is compiled out under coverage; tests and analyzeProject
// command reports still call the impl.
pub(crate) use lockfile_diff::lockfile_diff_json_impl;
#[cfg(feature = "mermaid-validation")]
pub(crate) use mermaid::validate_mermaid_markdown_json_impl;
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

json_binding!(
    dependencies_json,
    "dependenciesJson",
    dependencies_json_impl
);
json_binding!(dependents_json, "dependentsJson", dependents_json_impl);
json_binding!(related_json, "relatedJson", related_json_impl);
json_binding!(
    analyze_project_json,
    "analyzeProjectJson",
    analyze_project::analyze_project_value_impl,
    value
);

include!("napi_api/codebase_bindings.rs");

json_binding!(fetches_json, "fetchesJson", fetches_json_impl);
json_binding!(check_json, "checkJson", check_json_impl);
json_binding!(
    resolve_config_json,
    "resolveConfigJson",
    resolve_config_json_impl
);
#[cfg(feature = "mermaid-validation")]
json_binding!(
    validate_mermaid_markdown_json,
    "validateMermaidMarkdownJson",
    validate_mermaid_markdown_json_impl
);

include!("napi_api/planning_bindings.rs");

json_binding!(
    react_analyze_json,
    "reactAnalyzeJson",
    react_analyze_json_impl
);
json_binding!(react_check_json, "reactCheckJson", react_check_json_impl);
json_binding!(react_usages_json, "reactUsagesJson", react_usages_json_impl);

include!("napi_api/wrappers_query.rs");

json_binding!(
    lockfile_diff_json,
    "lockfileDiffJson",
    lockfile_diff_json_impl
);

include!("napi_api/ci_bindings.rs");
