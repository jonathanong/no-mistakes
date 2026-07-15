use super::legacy_test_support::{graph_report, import_usages_report, prepare_shared_traversal};
use super::*;
use crate::codebase::dependencies::Direction;
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

fn parser_fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/parser-count")
            .join(name),
    )
}

fn queue_fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/queue-ast-hop/basic/fixture"),
    )
}

#[test]
fn analyze_project_related_reports_require_files() {
    let cases = [
        (
            queue_fixture(),
            "queueRelated",
            "files must contain at least one file",
        ),
        (
            PathBuf::from(fixture_root("routes/good")),
            "serverRouteRelated",
            "files or roots must contain at least one entry",
        ),
        (
            parser_fixture("playwright"),
            "playwrightRelated",
            "files must contain at least one file",
        ),
    ];

    for (root, report_type, expected) in cases {
        let error = analyze_project_json_impl(
            json!({
                "root": root,
                "reports": [{ "type": report_type }]
            })
            .to_string(),
        )
        .unwrap_err();
        assert!(error.reason.contains(expected), "{report_type}: {error}");
    }
}

#[test]
fn analyze_project_dispatches_all_domain_report_types() {
    for report_type in [
        "symbols",
        "flow",
        "effects",
        "rscCallers",
        "importUsages",
        "queueEdges",
        "queueRelated",
        "queueCheck",
        "serverRoutes",
        "serverRouteList",
        "serverRouteEdges",
        "serverRouteRelated",
        "serverContracts",
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
fn analyze_project_dispatches_import_usages_report() {
    let output = analyze_project_json_impl(
        json!({
            "root": fixture_root("import-usages"),
            "reports": [{
                "type": "importUsages",
                "id": "imports",
                "filters": ["src/main.mts"]
            }]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();

    assert_eq!(value["reports"][0]["id"], "imports");
    assert_eq!(value["reports"][0]["type"], "importUsages");
    assert!(value["reports"][0]["result"]["files"][0]["imports"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["kind"] == "require-resolve" && row["packageName"] == "@scope/pkg"));
}

#[test]
fn import_usages_report_requires_shared_context() {
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": fixture_root("import-usages"),
            "reports": [{ "type": "importUsages", "filters": ["src/main.mts"] }]
        })
        .to_string(),
    )
    .unwrap();

    let error = import_usages_report(&options.reports[0], &options, None).unwrap_err();
    assert!(error.to_string().contains("without traversal context"));
}

#[test]
fn import_usages_report_reuses_shared_traversal_facts() {
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": fixture_root("import-usages"),
            "reports": [{ "type": "importUsages", "filters": ["src/main.mts"] }]
        })
        .to_string(),
    )
    .unwrap();
    let mut shared = prepare_shared_traversal(&options)
        .unwrap()
        .expect("import usages prepares shared traversal facts");

    let result = import_usages_report(&options.reports[0], &options, Some(&mut shared)).unwrap();

    assert!(result["files"][0]["imports"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["kind"] == "require-resolve" && row["packageName"] == "@scope/pkg"));
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

mod graph;
