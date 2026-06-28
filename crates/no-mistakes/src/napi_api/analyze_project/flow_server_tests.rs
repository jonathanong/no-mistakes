use super::*;
use serde_json::{json, Value};
use std::path::PathBuf;

fn fixture_root(name: &str) -> String {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis")
            .join(name)
            .join("fixture"),
    )
    .display()
    .to_string()
}

fn server_fixture_root(name: &str) -> String {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/server-ast-routes")
            .join(name)
            .join("fixture"),
    )
    .display()
    .to_string()
}

#[test]
fn analyze_project_dispatches_flow_report() {
    let output = analyze_project_json_impl(
        json!({
            "root": fixture_root("tests-impact-symbol"),
            "reports": [{
                "type": "flow",
                "id": "flow",
                "target": "utils.mts#parseDate",
                "direction": "both",
                "depth": 1,
                "relationships": ["import"]
            }]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();

    assert_eq!(value["reports"][0]["id"], "flow");
    assert_eq!(
        value["reports"][0]["result"]["target"],
        "utils.mts#parseDate"
    );
    assert!(!value["reports"][0]["result"]["nodes"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn flow_napi_rejects_unknown_direction() {
    let error = crate::napi_api::flow_json_impl(
        json!({
            "root": fixture_root("tests-impact-symbol"),
            "target": "utils.mts#parseDate",
            "direction": "sideways"
        })
        .to_string(),
    )
    .unwrap_err();

    assert!(error.reason.contains("unknown flow direction: sideways"));
}

#[test]
fn flow_napi_direct_impl_returns_report_and_validates_options() {
    let output = crate::napi_api::flow_json_impl(
        json!({
            "root": fixture_root("tests-impact-symbol"),
            "target": "utils.mts#parseDate",
            "direction": "deps",
            "depth": 1,
            "relationships": ["import"]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["target"], "utils.mts#parseDate");

    let missing_target = crate::napi_api::flow_json_impl(
        json!({
            "root": fixture_root("tests-impact-symbol")
        })
        .to_string(),
    )
    .unwrap_err();
    assert!(missing_target
        .reason
        .contains("target is required for flow"));

    let bad_relationship = crate::napi_api::flow_json_impl(
        json!({
            "root": fixture_root("tests-impact-symbol"),
            "target": "utils.mts#parseDate",
            "relationships": ["missing"]
        })
        .to_string(),
    )
    .unwrap_err();
    assert!(bad_relationship.reason.contains("unknown relationship"));
}

#[test]
fn analyze_project_dispatches_server_contracts_report() {
    let output = analyze_project_json_impl(
        json!({
            "root": server_fixture_root("express"),
            "reports": [{ "type": "serverContracts", "id": "contracts" }]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();

    assert_eq!(value["reports"][0]["id"], "contracts");
    assert!(value["reports"][0]["result"]["mismatches"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["missingParams"] == json!(["unused"])));
}

#[test]
fn server_contracts_napi_direct_impl_returns_report() {
    let output = crate::napi_api::server_contracts_json_impl(
        json!({
            "root": server_fixture_root("express")
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();

    assert!(value["routes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["route"] == "/api/v1/search"));

    let route_list = crate::napi_api::server_route_list_json_impl(
        json!({
            "root": server_fixture_root("express"),
            "files": ["/api/v1/search"]
        })
        .to_string(),
    )
    .unwrap();
    let routes: Value = serde_json::from_str(&route_list).unwrap();
    assert_eq!(routes.as_array().unwrap()[0]["route"], "/api/v1/search");
}

#[test]
fn server_contracts_napi_honors_roots_scope() {
    let output = crate::napi_api::server_contracts_json_impl(
        json!({
            "root": server_fixture_root("express"),
            "roots": ["backend/api/users.ts"]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();

    assert!(value["routes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["route"] == "/api/v1/search"));
    assert!(value["clientRefs"].as_array().unwrap().is_empty());
}

#[test]
fn tests_targets_napi_reports_project_commands() {
    let output = crate::napi_api::cli_parity::tests_targets_json_impl(
        json!({
            "root": fixture_root("test-plan-project-discovery"),
            "framework": "vitest",
            "files": ["web/storybook/button.stories.tsx"]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let targets = value["tests"][0]["targets"].as_array().unwrap();

    assert!(targets.iter().any(|target| target["project"] == "browser"));
    assert!(targets.iter().any(|target| target["project"] == "stories"));
}

#[test]
fn tests_targets_napi_rejects_missing_files() {
    for options in [
        json!({
            "root": fixture_root("test-plan-project-discovery"),
            "framework": "vitest"
        }),
        json!({
            "root": fixture_root("test-plan-project-discovery"),
            "framework": "vitest",
            "files": []
        }),
    ] {
        let error = crate::napi_api::cli_parity::tests_targets_json_impl(options.to_string())
            .expect_err("missing files should fail");
        assert!(error.reason.contains("files is required"));
    }
}
