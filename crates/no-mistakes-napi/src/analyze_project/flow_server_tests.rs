use super::*;
use serde_json::{json, Value};
use std::path::PathBuf;

fn fixture_root(name: &str) -> String {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis")
            .join(name)
            .join("fixture"),
    )
    .display()
    .to_string()
}

fn server_fixture_root(name: &str) -> String {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/server-ast-routes")
            .join(name)
            .join("fixture"),
    )
    .display()
    .to_string()
}

fn parser_count_server_fixture() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/parser-count/server-contracts"),
    )
}

include!("flow_server_tests/flow.rs");
include!("flow_server_tests/server.rs");
include!("flow_server_tests/targets.rs");
include!("flow_server_tests/vitest_workspace_precedence.rs");
