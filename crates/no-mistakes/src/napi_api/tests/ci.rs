use crate::napi_api::{
    ci_env_json_impl, ci_impact_json_impl, ci_topology_json_impl, impacted_checks_json_impl,
};
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

fn workflow_topology_fixture(name: &str) -> String {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/workflow-topology")
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

fn impacted_checks_multi_framework_root() -> String {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/impacted-checks/multi-framework"),
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

#[test]
fn impacted_checks_json_timings_are_opt_in_and_ordered() {
    let root = impacted_checks_multi_framework_root();
    let changed_files = [
        "src/value.ts",
        "dotnet/src/App/Value.cs",
        "swift/App/Sources/App/Value.swift",
    ];
    let plain_options = json!({
        "root": root,
        "changedFiles": changed_files,
    })
    .to_string();
    let timed_options = json!({
        "root": impacted_checks_multi_framework_root(),
        "changedFiles": changed_files,
        "timings": true,
    })
    .to_string();

    let mut plain: serde_json::Value =
        serde_json::from_str(&impacted_checks_json_impl(plain_options).unwrap()).unwrap();
    assert!(plain.get("timings").is_none());
    let mut timed: serde_json::Value =
        serde_json::from_str(&impacted_checks_json_impl(timed_options).unwrap()).unwrap();
    let timings = timed
        .get("timings")
        .and_then(serde_json::Value::as_array)
        .expect("timings should be returned when requested");
    assert_eq!(
        timings
            .iter()
            .map(|timing| timing["phase"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec![
            "prepare",
            "discover.dotnet",
            "discover.vitest",
            "discover.playwright",
            "discover.swift",
            "graph",
            "select.dotnet",
            "select.vitest",
            "select.playwright",
            "select.swift",
            "generic-checks",
            "total",
        ]
    );
    assert!(timings
        .iter()
        .all(|timing| timing["duration_ms"].as_f64().is_some()));
    let total = timings
        .last()
        .and_then(|timing| timing["duration_ms"].as_f64())
        .unwrap();
    let phase_sum: f64 = timings[..timings.len() - 1]
        .iter()
        .map(|timing| timing["duration_ms"].as_f64().unwrap())
        .sum();
    assert!(
        phase_sum <= total + 0.001,
        "exclusive phase durations must not double-count nested graph work: {phase_sum} > {total}"
    );

    timed.as_object_mut().unwrap().remove("timings");
    plain.as_object_mut().unwrap().remove("timings");
    assert_eq!(timed, plain);
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

#[test]
fn ci_topology_json_returns_the_parsed_graph() {
    let options = json!({ "root": workflow_topology_fixture("needs-basic") }).to_string();
    let output = ci_topology_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["schemaVersion"], 1);
    assert_eq!(value["workflows"].as_array().unwrap().len(), 1);
    assert_eq!(value["jobs"].as_array().unwrap().len(), 3);
    assert!(value["diagnostics"].as_array().unwrap().is_empty());
}

#[test]
fn ci_topology_json_reports_diagnostics_without_failing() {
    // Unlike the CLI (which exits non-zero and prints nothing on error
    // diagnostics), the N-API entry point always returns the full graph —
    // callers decide what to do with a non-empty `diagnostics` array.
    let options = json!({ "root": workflow_topology_fixture("job-cycle") }).to_string();
    let output = ci_topology_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    let diagnostics = value["diagnostics"].as_array().unwrap();
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic["code"] == "job-dependency-cycle"));
}

#[test]
fn ci_topology_json_applies_workflow_filter() {
    let options = json!({
        "root": workflow_topology_fixture("workflow-filters"),
        "workflows": ["root.yml"],
    })
    .to_string();
    let output = ci_topology_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    let paths: Vec<&str> = value["workflows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["path"].as_str().unwrap())
        .collect();
    assert_eq!(
        paths,
        vec![".github/workflows/callee.yml", ".github/workflows/root.yml"]
    );
}

#[test]
fn ci_topology_json_defaults_root() {
    assert!(ci_topology_json_impl(json!({}).to_string()).is_ok());
}
