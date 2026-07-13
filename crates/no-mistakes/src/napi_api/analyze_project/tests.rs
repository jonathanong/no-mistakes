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
fn analyze_project_queue_views_share_one_report_and_parse_pass() {
    let source = queue_fixture();
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let root_json = root.display().to_string();
    let standalone = [
        crate::napi_api::queues_json_impl(json!({ "root": root_json }).to_string()).unwrap(),
        crate::napi_api::queue_edges_json_impl(
            json!({ "root": root_json, "files": ["enqueue.ts"], "depth": 2 }).to_string(),
        )
        .unwrap(),
        crate::napi_api::queue_related_json_impl(
            json!({ "root": root_json, "files": ["enqueue.ts"], "direction": "deps" }).to_string(),
        )
        .unwrap(),
        crate::napi_api::queue_check_json_impl(json!({ "root": root_json }).to_string()).unwrap(),
    ]
    .map(|value| serde_json::from_str::<Value>(&value).unwrap());

    crate::ast::begin_parse_count(&root);
    let output = analyze_project_json_impl(
        json!({
            "root": root_json,
            "reports": [
                { "type": "queues" },
                { "type": "queueEdges", "files": ["enqueue.ts"], "depth": 2 },
                { "type": "queueRelated", "files": ["enqueue.ts"], "direction": "deps" },
                { "type": "queueCheck" }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    let value: Value = serde_json::from_str(&output).unwrap();

    for (index, expected) in standalone.iter().enumerate() {
        assert_eq!(&value["reports"][index]["result"], expected);
    }
    assert!(!counts.is_empty(), "queue fixture must exercise the parser");
    assert!(counts.values().all(|count| *count == 1), "{counts:#?}");
}

#[test]
fn analyze_project_playwright_views_share_one_analysis_with_standalone_parity() {
    let root = parser_fixture("playwright");
    let root_json = root.display().to_string();
    let standalone = [
        crate::napi_api::playwright_check_json_impl(json!({ "root": root_json }).to_string())
            .unwrap(),
        crate::napi_api::playwright_edges_json_impl(json!({ "root": root_json }).to_string())
            .unwrap(),
        crate::napi_api::playwright_related_json_impl(
            json!({ "root": root_json, "files": ["app/page.tsx"] }).to_string(),
        )
        .unwrap(),
        crate::napi_api::playwright_tests_json_impl(
            json!({ "root": root_json, "files": ["app/page.tsx"] }).to_string(),
        )
        .unwrap(),
    ]
    .map(|value| serde_json::from_str::<Value>(&value).unwrap());

    let output = analyze_project_json_impl(
        json!({
            "root": root_json,
            "reports": [
                { "type": "playwrightCheck" },
                { "type": "playwrightEdges" },
                { "type": "playwrightRelated", "files": ["app/page.tsx"] },
                { "type": "playwrightTests", "files": ["app/page.tsx"] }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();

    for (index, expected) in standalone.iter().enumerate() {
        assert_eq!(&value["reports"][index]["result"], expected);
    }
}

#[test]
fn analyze_project_server_views_share_one_report_with_standalone_parity() {
    let root = fixture_root("routes/good");
    let standalone = [
        crate::napi_api::server_routes_json_impl(json!({ "root": root }).to_string()).unwrap(),
        crate::napi_api::server_route_list_json_impl(
            json!({ "root": root, "files": ["/api/v1/users"] }).to_string(),
        )
        .unwrap(),
        crate::napi_api::server_route_edges_json_impl(
            json!({ "root": root, "files": ["backend/api/v1/users.mts"] }).to_string(),
        )
        .unwrap(),
        crate::napi_api::server_route_related_json_impl(
            json!({
                "root": root,
                "roots": ["backend/api/v1/users.mts"],
                "direction": "dependents"
            })
            .to_string(),
        )
        .unwrap(),
    ]
    .map(|value| serde_json::from_str::<Value>(&value).unwrap());

    let output = analyze_project_json_impl(
        json!({
            "root": root,
            "reports": [
                { "type": "serverRoutes" },
                { "type": "serverRouteList", "files": ["/api/v1/users"] },
                { "type": "serverRouteEdges", "files": ["backend/api/v1/users.mts"] },
                {
                    "type": "serverRouteRelated",
                    "roots": ["backend/api/v1/users.mts"],
                    "direction": "dependents"
                }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();

    for (index, expected) in standalone.iter().enumerate() {
        assert_eq!(&value["reports"][index]["result"], expected);
    }
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
fn graph_reports_honor_per_report_scope_overrides() {
    let output = analyze_project_json_impl(
        json!({
            "root": fixture_root("exports"),
            "reports": [{
                "type": "dependencies",
                "root": fixture_root("simple"),
                "files": ["a.mts"]
            }]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    assert!(value["reports"][0]["result"]["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|file| file["path"] == "b.mts"));
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

include!("tests/symbol_reports.rs");
