use crate::napi_api::{ci_env_json_impl, ci_impact_json_impl, impacted_checks_json_impl};
use serde_json::json;
use std::path::PathBuf;

fn ci_graph(name: &str) -> String {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/ci-graph")
            .join(name),
    )
    .display()
    .to_string()
}

fn impacted_checks_root() -> String {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/impacted-checks/basic"),
    )
    .display()
    .to_string()
}

#[test]
fn ci_impact_json_returns_workflows() {
    let options = json!({ "root": ci_graph("triggers"), "files": ["src/app.ts"] }).to_string();
    let output = ci_impact_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(!value["workflows"].as_array().unwrap().is_empty());
}

#[test]
fn ci_env_json_returns_locations() {
    let options = json!({ "root": ci_graph("env"), "var": "CIGRAPH_WORKFLOW_VAR" }).to_string();
    let output = ci_env_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["variable"], "CIGRAPH_WORKFLOW_VAR");
    assert!(!value["files"].as_array().unwrap().is_empty());
}

#[test]
fn ci_env_json_requires_var() {
    let options = json!({ "root": ci_graph("env") }).to_string();
    assert!(ci_env_json_impl(options).is_err());
}

#[test]
fn impacted_checks_json_returns_checks() {
    let options =
        json!({ "root": impacted_checks_root(), "changedFiles": ["src/foo.ts"] }).to_string();
    let output = impacted_checks_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(value["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|check| check["name"] == "vitest"));
}

// Omitting `root` exercises the default-root (`"."`) fallback in each entry
// point. The crate dir has no workflows/config, so results are empty but valid.
#[test]
fn ci_impact_json_defaults_root() {
    assert!(ci_impact_json_impl(json!({ "files": [] }).to_string()).is_ok());
}

#[test]
fn ci_env_json_defaults_root() {
    assert!(ci_env_json_impl(json!({ "var": "X" }).to_string()).is_ok());
}

#[test]
fn impacted_checks_json_defaults_root() {
    assert!(impacted_checks_json_impl(json!({}).to_string()).is_ok());
}
