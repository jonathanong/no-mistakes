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

fn check_runner_fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/check-runner")
            .join(name)
            .join("fixture"),
    )
}

fn analyze_project_check_result(root: &PathBuf) -> Value {
    let output = analyze_project_json_impl(
        json!({
            "root": root,
            "config": ".no-mistakes.yml",
            "reports": [{ "type": "check" }]
        })
        .to_string(),
    )
    .unwrap();
    serde_json::from_str::<Value>(&output).unwrap()["reports"][0]["result"].clone()
}

fn standalone_check_result(root: &PathBuf) -> Value {
    serde_json::from_str(
        &crate::napi_api::check_json_impl(
            json!({ "root": root, "config": ".no-mistakes.yml" }).to_string(),
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn analyze_project_value_impl_accepts_parsed_options() {
    let output = analyze_project_value_impl(json!({
        "root": fixture_root("simple"),
        "reports": []
    }))
    .unwrap();

    assert_eq!(
        serde_json::from_str::<Value>(&output).unwrap()["reports"],
        json!([])
    );
}

#[test]
fn analyze_project_dynamic_import_check_respects_filesystem_skips_with_standalone_parity() {
    let root = check_runner_fixture("dynamic-import-respects-filesystem-skip");
    let result = analyze_project_check_result(&root);

    assert_eq!(result, standalone_check_result(&root));
    let finding = result["rules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|finding| finding["file"] == "tests/scoped.test.ts")
        .expect("skipped dynamic import remains reportable as unresolved");
    assert_eq!(finding["import"], "../skipped/target");
    assert!(finding.get("target").is_none());
}

#[test]
fn analyze_project_check_applies_shared_suppression_accounting() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check/suppression-react");
    let output = analyze_project_json_impl(
        json!({
            "root": root,
            "reports": [{ "type": "check", "includeSuppressed": true }]
        })
        .to_string(),
    )
    .unwrap();
    let result: Value = serde_json::from_str(&output).unwrap();
    let report = &result["reports"][0]["result"];
    assert!(report["react"].as_array().is_some_and(Vec::is_empty));
    assert!(report["suppressed"].as_array().is_some_and(|items| {
        items
            .iter()
            .any(|item| item["domain"] == "react" && item["rule"] == "assert-no-fetch")
    }));
}

#[test]
fn analyze_project_reachability_check_uses_full_graph_with_standalone_parity() {
    let root = check_runner_fixture("required-reachability-ignores-filesystem-skip");
    let result = analyze_project_check_result(&root);

    assert_eq!(result, standalone_check_result(&root));
    assert!(result["rules"].as_array().unwrap().iter().any(|finding| {
        finding["rule"] == "required-entrypoint-reachability"
            && finding["file"] == "sources/unreachable.ts"
            && finding["message"]
                .as_str()
                .is_some_and(|message| message.contains("not runtime-reachable"))
    }));
}

#[test]
fn analyze_project_check_and_graph_report_share_one_canonical_graph() {
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": fixture_root("test-no-unmocked-dynamic-imports"),
            "config": ".no-mistakes-combined.yml",
            "reports": [
                { "type": "check" },
                {
                    "type": "dependents",
                    "files": ["src/unmocked-child.mts"],
                    "relationships": ["import"]
                }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let mut context = context::AnalyzeProjectContext::prepare(&options).unwrap();
    for request in &options.reports {
        run_report(request, &options, &mut context).unwrap();
    }
    assert_eq!(context.graph_build_count(), 1);
}

#[test]
fn mixed_check_keeps_non_import_edges_from_the_union_graph() {
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": fixture_root("codebase-intel"),
            "config": ".no-mistakes-union-graph.yml",
            "reports": [
                { "type": "check" },
                {
                    "type": "dependencies",
                    "files": ["packages/api/src/send-email.mts"],
                    "relationships": ["queue"],
                    "depth": 1
                }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let mut context = context::AnalyzeProjectContext::prepare(&options).unwrap();
    let results = options
        .reports
        .iter()
        .map(|request| run_report(request, &options, &mut context).unwrap())
        .collect::<Vec<_>>();

    assert!(
        results[1]["files"].as_array().unwrap().iter().any(|entry| {
            entry["queueFile"] == "packages/api/src/emails.mts"
                && entry["job"] == "sendWelcomeEmail"
                && entry["via"]
                    .as_array()
                    .is_some_and(|via| via.iter().any(|kind| kind == "queue-enqueue"))
        }),
        "queue edge missing from union graph: {:#?}",
        results[1]
    );
    assert_eq!(context.graph_build_count(), 1);
}

#[test]
fn mixed_check_collects_ignored_explicit_graph_root_facts() {
    let fixture = crate::test_support::materialize_gitignore_fixture("prepared-tsconfig");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": root,
            "reports": [
                { "type": "check" },
                {
                    "type": "dependencies",
                    "files": ["ignored-explicit/effect-entry.ts"],
                    "relationships": ["import"],
                    "depth": 1
                }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let mut context = context::AnalyzeProjectContext::prepare(&options).unwrap();
    let results = options
        .reports
        .iter()
        .map(|request| run_report(request, &options, &mut context).unwrap())
        .collect::<Vec<_>>();

    assert!(
        results[1]["files"].as_array().unwrap().iter().any(|entry| {
            entry["path"] == "src/effect.ts"
                && entry["via"]
                    .as_array()
                    .is_some_and(|via| via.iter().any(|kind| kind == "import"))
        }),
        "ignored explicit root was not analyzed: {:#?}",
        results[1]
    );
    assert_eq!(context.graph_build_count(), 1);
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
