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

#[test]
fn analyze_project_batches_graph_and_queue_reports() {
    let output = analyze_project_json_impl(
        json!({
            "root": fixture_root("simple"),
            "reports": [
                { "type": "dependencies", "id": "deps", "files": ["a.mts"], "relationships": ["import"] },
                { "type": "dependents", "id": "users", "files": ["b.mts"], "relationships": ["import"] },
                { "type": "queues" }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["reports"][0]["id"], "deps");
    assert_eq!(value["reports"][0]["type"], "dependencies");
    assert!(value["reports"][0]["result"]["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|file| file["path"] == "b.mts"));
    assert_eq!(value["reports"][1]["id"], "users");
    assert_eq!(value["reports"][2]["type"], "queues");
}

#[test]
fn analyze_project_dispatches_all_domain_report_types() {
    for report_type in [
        "symbols",
        "queueEdges",
        "queueRelated",
        "queueCheck",
        "serverRoutes",
        "serverRouteList",
        "serverRouteEdges",
        "serverRouteRelated",
        "reactAnalyze",
        "reactCheck",
        "playwrightCheck",
        "playwrightEdges",
        "playwrightRelated",
        "playwrightTests",
        "check",
    ] {
        let result = analyze_project_json_impl(
            json!({
                "root": fixture_root("simple"),
                "reports": [{
                    "type": report_type,
                    "id": report_type,
                    "files": ["a.mts"]
                }]
            })
            .to_string(),
        );
        if let Err(error) = result {
            assert!(
                !error.reason.contains("unknown analyzeProject report type"),
                "{report_type} should be recognized"
            );
        }
    }
}

#[test]
fn analyze_project_rejects_unknown_report_type() {
    let error = analyze_project_json_impl(
        json!({
            "root": fixture_root("simple"),
            "reports": [{ "type": "missing" }]
        })
        .to_string(),
    )
    .unwrap_err();
    assert!(error.reason.contains("unknown analyzeProject report type"));
}

#[test]
fn prepare_shared_traversal_skips_non_graph_reports() {
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": fixture_root("simple"),
            "reports": [{ "type": "queues" }]
        })
        .to_string(),
    )
    .unwrap();
    assert!(prepare_shared_traversal(&options).unwrap().is_none());
}

#[test]
fn graph_report_requires_shared_context() {
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": fixture_root("simple"),
            "reports": [{ "type": "dependencies", "files": ["a.mts"] }]
        })
        .to_string(),
    )
    .unwrap();
    let error = graph_report(&options.reports[0], &options, Direction::Deps, None).unwrap_err();
    assert!(error.to_string().contains("without traversal context"));
}

#[test]
fn graph_reports_reject_per_report_scope_overrides() {
    let error = analyze_project_json_impl(
        json!({
            "root": fixture_root("simple"),
            "reports": [{
                "type": "dependencies",
                "root": fixture_root("simple"),
                "files": ["a.mts"]
            }]
        })
        .to_string(),
    )
    .unwrap_err();
    assert!(error.reason.contains("per-report root/tsconfig"));
}

#[test]
fn shared_graph_context_builds_once_for_multiple_graph_reports() {
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": fixture_root("simple"),
            "reports": [
                { "type": "dependencies", "files": ["a.mts"], "relationships": ["import"] },
                { "type": "dependents", "files": ["b.mts"], "relationships": ["import"] }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let mut shared = prepare_shared_traversal(&options).unwrap().unwrap();
    for request in &options.reports {
        let direction = if request.report_type == "dependencies" {
            Direction::Deps
        } else {
            Direction::Dependents
        };
        let _ = graph_report(request, &options, direction, Some(&mut shared)).unwrap();
    }
    assert_eq!(shared.graph_builds, 1);
}

#[test]
fn shared_graph_context_supports_symbol_dependents() {
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": fixture_root("simple"),
            "reports": [
                { "type": "dependents", "files": ["b.mts#b"], "relationships": ["import"] }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let mut shared = prepare_shared_traversal(&options).unwrap().unwrap();
    let result = graph_report(
        &options.reports[0],
        &options,
        Direction::Dependents,
        Some(&mut shared),
    )
    .unwrap();
    assert_eq!(shared.graph_builds, 1);
    assert!(result["files"].is_array());
}
