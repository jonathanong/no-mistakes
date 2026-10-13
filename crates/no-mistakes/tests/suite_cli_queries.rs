//! Cli queries integration tests, linked in one harness.

// Share stateless fixture helpers across the modules in this harness.
#[path = "common/gitignore_fixture.rs"]
mod gitignore_fixture;

#[path = "cli.rs"]
mod cli;
#[path = "cli_ci.rs"]
mod cli_ci;
#[path = "cli_codebase_acceptance.rs"]
mod cli_codebase_acceptance;
#[path = "cli_config_resolve.rs"]
mod cli_config_resolve;
#[path = "cli_diagnostics.rs"]
mod cli_diagnostics;
#[path = "cli_extra.rs"]
mod cli_extra;
#[path = "cli_extra2.rs"]
mod cli_extra2;
#[path = "cli_flow.rs"]
mod cli_flow;
#[path = "cli_format.rs"]
mod cli_format;
#[path = "cli_format2.rs"]
mod cli_format2;
#[path = "cli_import_usages.rs"]
mod cli_import_usages;
#[path = "cli_infra_swift.rs"]
mod cli_infra_swift;
#[path = "cli_integration_route_coverage.rs"]
mod cli_integration_route_coverage;
#[path = "cli_invocation.rs"]
mod cli_invocation;
#[path = "cli_lockfile.rs"]
mod cli_lockfile;
#[path = "cli_lockfile2.rs"]
mod cli_lockfile2;
#[path = "cli_markdown_reachability_depth_three.rs"]
mod cli_markdown_reachability_depth_three;
#[path = "cli_proxy.rs"]
mod cli_proxy;
#[path = "cli_queries.rs"]
mod cli_queries;
#[path = "cli_react_usages.rs"]
mod cli_react_usages;
#[path = "cli_server.rs"]
mod cli_server;
#[path = "cli_server_lang.rs"]
mod cli_server_lang;
#[path = "cli_typescript_paths_precedence.rs"]
mod cli_typescript_paths_precedence;
#[path = "cli_unique_exports.rs"]
mod cli_unique_exports;
