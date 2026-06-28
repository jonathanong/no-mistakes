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
        "importUsages",
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
fn graph_reports_surface_traversal_errors() {
    let error = analyze_project_json_impl(
        json!({
            "root": fixture_root("simple"),
            "filters": ["["],
            "reports": [{ "type": "dependencies", "files": ["a.mts"] }]
        })
        .to_string(),
    )
    .unwrap_err();
    assert!(error.reason.contains("glob"));
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
fn shared_graph_context_keeps_import_only_dependencies_lazy() {
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": fixture_root("simple"),
            "reports": [
                { "type": "dependencies", "files": ["a.mts"], "relationships": ["import"] }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let mut shared = prepare_shared_traversal(&options).unwrap().unwrap();
    let result = graph_report(
        &options.reports[0],
        &options,
        Direction::Deps,
        Some(&mut shared),
    )
    .unwrap();
    assert_eq!(shared.graph_builds, 0);
    assert!(result["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|file| { file["path"] == "b.mts" }));
}

#[test]
fn shared_import_usages_context_reuses_collected_facts() {
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": fixture_root("import-usages"),
            "reports": [
                { "type": "importUsages", "filters": ["src/main.mts"] }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let mut shared = prepare_shared_traversal(&options).unwrap().unwrap();
    assert!(!shared.facts().is_empty());
    assert!(!shared.facts().is_empty());
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

#[test]
fn analyze_project_shared_dependencies_uses_symbol_graph_when_included() {
    let output = analyze_project_json_impl(
        json!({
            "root": fixture_root("tests-impact-symbol"),
            "reports": [{
                "type": "dependencies",
                "id": "deps",
                "includeSymbols": true,
                "relationships": ["import"],
                "files": [{ "file": "other.mts", "symbol": "parse" }]
            }]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let files = value["reports"][0]["result"]["files"].as_array().unwrap();

    assert!(files
        .iter()
        .any(|file| file["file"] == "utils.mts" && file["symbol"] == "parseDate"));
}

#[test]
fn analyze_project_shared_symbol_graph_does_not_leak_into_plain_reports() {
    let output = analyze_project_json_impl(
        json!({
            "root": fixture_root("tests-impact-symbol"),
            "reports": [
                {
                    "type": "dependencies",
                    "id": "symbol-deps",
                    "includeSymbols": true,
                    "relationships": ["import"],
                    "files": [{ "file": "other.mts", "symbol": "parse" }]
                },
                {
                    "type": "dependencies",
                    "id": "plain-deps",
                    "relationships": ["import"],
                    "files": ["other.mts"]
                }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let files = value["reports"][1]["result"]["files"].as_array().unwrap();

    assert!(files.iter().any(|file| file["path"] == "utils.mts"));
    assert!(!files.iter().any(|file| file.get("symbol").is_some()));
}

#[test]
fn analyze_project_shared_dependents_uses_symbol_graph_when_included() {
    let output = analyze_project_json_impl(
        json!({
            "root": fixture_root("tests-impact-symbol"),
            "reports": [{
                "type": "dependents",
                "id": "users",
                "includeSymbols": true,
                "relationships": ["import"],
                "files": [{ "file": "utils.mts", "symbol": "parseDate" }]
            }]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let files = value["reports"][0]["result"]["files"].as_array().unwrap();

    assert!(files
        .iter()
        .any(|file| file["file"] == "other.mts" && file["symbol"] == "parse"));
    assert!(!files
        .iter()
        .any(|file| file["path"] == "unrelated-consumer.mts"));
}

#[test]
fn analyze_project_dispatches_signature_impact_symbols_report() {
    let output = analyze_project_json_impl(
        json!({
            "root": fixture_root("tests-impact-symbol"),
            "reports": [{
                "type": "symbols",
                "id": "impact",
                "files": ["utils.mts"],
                "mode": "signature-impact",
                "symbol": "parseDate"
            }]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let result = &value["reports"][0]["result"];

    assert_eq!(value["reports"][0]["id"], "impact");
    assert_eq!(result["symbol"], "parseDate");
    assert!(result["testCallers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| { entry["file"] == "helper-export.test.mts" }));
}

#[test]
fn analyze_project_shared_symbol_dependents_expands_file_roots() {
    let output = analyze_project_json_impl(
        json!({
            "root": fixture_root("symbol-export"),
            "reports": [{
                "type": "dependents",
                "id": "users",
                "includeSymbols": true,
                "relationships": ["import"],
                "files": ["file-root-source.mts"]
            }]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let files = value["reports"][0]["result"]["files"].as_array().unwrap();

    assert!(files
        .iter()
        .any(|file| file["path"] == "file-root-consumer.mts"));
    assert!(files
        .iter()
        .any(|file| file["file"] == "file-root-consumer.mts" && file["symbol"] == "value"));
}

#[test]
fn tests_impact_api_requires_entrypoints() {
    let error = crate::napi_api::cli_parity::build_impact_args(
        crate::napi_api::options::TestsImpactOptions {
            entrypoints: vec![],
            include_symbols: false,
            root: None,
            config: None,
            tsconfig: None,
        },
    )
    .unwrap_err();

    assert!(error.to_string().contains("entrypoints is required"));
}
