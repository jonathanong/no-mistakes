#![cfg_attr(
    all(coverage, not(test)),
    expect(
        dead_code,
        unused_imports,
        reason = "Coverage omits native registration while retaining the Rust API implementations."
    )
)]

#[cfg(not(coverage))]
use napi::bindgen_prelude::AsyncTask;
#[cfg(all(not(test), not(coverage)))]
use napi_derive::napi;

#[macro_use]
mod binding_macro;

mod analyze_project;
#[cfg(feature = "test-instrumentation")]
pub(crate) use analyze_project::analyze_project_json_impl;
mod async_task;
mod cli_parity;
pub(crate) mod codebase;
mod diagnostics;
pub(crate) use diagnostics::diagnostics_snapshot_impl;
#[cfg(not(coverage))]
pub use diagnostics::diagnostics_snapshot_json;
mod changes_diff;
mod filesystem;
pub(crate) mod infra_swift;
mod lock_status;
pub(crate) use queue::{
    queue_check_json_impl, queue_edges_json_impl, queue_related_json_impl, queues_json_impl,
};

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod lock_status_tests;
mod lockfile_diff;
#[cfg(not(coverage))]
pub(crate) use changes_diff::changes_diff_json_impl;
#[cfg(feature = "mermaid-validation")]
mod mermaid;
pub(crate) mod options;
mod postgres_source;
mod typescript_modules;
#[cfg(not(coverage))]
pub use postgres_source::extract_executed_postgres_sql_json;
#[cfg(not(coverage))]
pub use postgres_source::parse_postgres_sql_json;
#[cfg(not(coverage))]
pub use typescript_modules::analyze_typescript_modules_json;
mod project;
pub mod queries;
mod queue;

#[cfg(not(coverage))]
use async_task::{JsonValueTask, VersionTask};
pub(crate) use cli_parity::{
    check_json_impl, ci_env_json_impl, ci_impact_json_impl, ci_topology_impact_json_impl,
    ci_topology_json_impl, fetches_json_impl, impacted_checks_json_impl,
    playwright_check_json_impl, playwright_edges_json_impl, playwright_related_json_impl,
    playwright_tests_json_impl, resolve_config_json_impl, tests_audit_json_impl,
    tests_comment_markdown_impl, tests_graph_json_impl, tests_graph_mermaid_impl,
    tests_impact_json_impl, tests_plan_json_impl, tests_targets_json_impl, tests_why_json_impl,
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
pub(crate) use lock_status::lock_status_json_impl;
pub(crate) use lockfile_diff::lockfile_diff_json_impl;
#[cfg(feature = "mermaid-validation")]
pub(crate) use mermaid::validate_mermaid_markdown_json_impl;
pub(crate) use project::{
    data_pw_json_impl, effects_json_impl, flow_json_impl, react_analyze_json_impl,
    react_check_json_impl, react_usages_json_impl, registry_extension_json_impl,
    rsc_callers_json_impl, server_contracts_json_impl, server_route_edges_json_impl,
    server_route_list_json_impl, server_route_related_json_impl, server_routes_json_impl,
};

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests_check_images;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests_check_timing;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests_queue_waits;

include!("napi_api/primitive_bindings.rs");

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
    lock_status_json,
    "lockStatusJson",
    lock_status_json_impl,
    pure
);
json_binding!(
    lockfile_diff_json,
    "lockfileDiffJson",
    lockfile_diff_json_impl
);

include!("napi_api/ci_bindings.rs");

include!("napi_api/postgres_bindings.rs");

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
#[path = "napi_api/suppression_index_tests.rs"]
mod suppression_index_tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests_queue_factory_wrappers;
