use std::path::PathBuf;

use napi::{Env, Task};
use serde_json::json;

use super::async_task::{JsonTask, VersionTask};
use super::options::{
    parse_export_kind, parse_include, parse_options, parse_queue_direction, parse_relationship,
    parse_server_direction, project_roots, resolve_project_root, ProjectOptions, SymbolOptions,
    TraverseOptions,
};
use super::*;

include!("tests_symbols_impact.rs");
include!("tests/tests_native_fallback.rs");

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

fn fixture(category: &str, name: &str) -> String {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases")
            .join(category)
            .join(name)
            .join("fixture"),
    )
    .display()
    .to_string()
}

fn saved_fixture(path: &[&str]) -> String {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    root.extend(path);
    crate::codebase::ts_resolver::normalize_path(&root)
        .display()
        .to_string()
}

#[test]
fn version_returns_crate_version() {
    assert_eq!(version_impl(), env!("CARGO_PKG_VERSION"));
}

fn echo_task(input: String) -> napi::Result<String> {
    Ok(format!("echo:{input}"))
}

fn failing_task(_input: String) -> napi::Result<String> {
    Err(napi::Error::from_reason("task failed"))
}

#[test]
fn async_json_task_runs_on_task_interface() {
    let mut task = JsonTask::new("{}".to_string(), echo_task);

    assert_eq!(task.compute().unwrap(), "echo:{}");
    assert_eq!(
        task.resolve(Env::from_raw(std::ptr::null_mut()), "done".to_string())
            .unwrap(),
        "done"
    );

    let mut task = JsonTask::new("{}".to_string(), failing_task);
    assert!(task.compute().unwrap_err().reason.contains("task failed"));
}

#[test]
fn async_version_task_runs_on_task_interface() {
    let mut task = VersionTask;

    assert_eq!(task.compute().unwrap(), env!("CARGO_PKG_VERSION"));
    assert_eq!(
        task.resolve(Env::from_raw(std::ptr::null_mut()), "0.0.0".to_string())
            .unwrap(),
        "0.0.0"
    );
}

#[test]
fn dependencies_json_returns_structured_results() {
    let options = json!({
        "root": fixture_root("simple"),
        "files": ["a.mts"],
        "relationships": ["import"]
    })
    .to_string();

    let output = dependencies_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(value["roots"], json!(["a.mts"]));
    assert!(value["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|file| file["path"] == "b.mts"));
}

#[test]
fn dependents_json_returns_structured_results() {
    let options = json!({
        "root": fixture_root("simple"),
        "files": ["b.mts"],
        "relationships": ["import"]
    })
    .to_string();

    let output = dependents_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(value["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|file| file["path"] == "a.mts"));
}

#[test]
fn related_json_matches_dependents_alias() {
    let options = json!({
        "root": fixture_root("simple"),
        "files": ["b.mts"],
        "relationships": ["import"]
    })
    .to_string();

    let output = related_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(value["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|file| file["path"] == "a.mts"));
}

#[test]
fn symbols_json_returns_structured_results() {
    let options = json!({
        "root": fixture_root("symbols-output"),
        "files": ["src/utils.mts"],
        "include": "both"
    })
    .to_string();

    let output = symbols_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(value["roots"], json!(["src/utils.mts"]));
    assert_eq!(value["files"][0]["path"], "src/utils.mts");
}

#[test]
fn pass4b_symbols_cli_and_napi_reports_share_gitignore_visibility() {
    let fixture = crate::test_support::materialize_gitignore_fixture("pass4b-shadow");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let root_string = root.display().to_string();

    let cli_output = crate::codebase::symbols::run_json(crate::codebase::symbols::SymbolsArgs {
        files: vec![PathBuf::from("query/source.ts")],
        root: Some(root),
        tsconfig: None,
        config: None,
        mode: crate::codebase::symbols::SymbolsMode::List,
        symbol: None,
        kinds: Vec::new(),
        include: crate::codebase::symbols::Include::Both,
        format: Some(crate::cli::Format::Json),
        json: true,
        timings: false,
    })
    .unwrap();
    let napi_output = symbols_json_impl(
        json!({
            "root": root_string,
            "files": ["query/source.ts"],
            "include": "both",
        })
        .to_string(),
    )
    .unwrap();
    let cli_output: serde_json::Value = serde_json::from_str(&cli_output).unwrap();
    let napi_output: serde_json::Value = serde_json::from_str(&napi_output).unwrap();

    assert_eq!(napi_output, cli_output);
    let reexports: Vec<_> = napi_output["files"][0]["exports"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|export| export["kind"] == "re-export")
        .collect();
    assert!(!reexports.is_empty());
    assert!(
        reexports
            .iter()
            .all(|export| export["reExport"]["resolved"] == "query/target.ts"),
        "unexpected re-export rows: {reexports:#?}"
    );
    assert!(napi_output["files"][0]["imports"]
        .as_array()
        .unwrap()
        .iter()
        .all(|import| import["resolved"] == "query/target.ts"));
}

#[test]
fn fetches_json_returns_structured_report() {
    let options = json!({ "root": fixture("nextjs-fetches", "next-app") }).to_string();
    let output = fetches_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(value["summary"]["totalRoutes"].as_u64().unwrap() > 0);
    assert!(value["routes"].as_array().unwrap().iter().any(|route| {
        route["apiCalls"]
            .as_array()
            .is_some_and(|calls| !calls.is_empty())
    }));
}

include!("tests_planning.rs");

#[test]
fn tests_plan_json_ignores_deleted_changed_files() {
    let output = tests_plan_json_impl(
        json!({
            "framework": "vitest",
            "root": fixture_root("test-plan-config"),
            "changedFiles": ["web/app/deleted.tsx", "source.ts"],
            "limitFiles": 1
        })
        .to_string(),
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(plan["fallback_triggered"], false);
    assert!(plan["fallback_reason"].is_null());
    assert!(plan["warnings"].as_array().unwrap().is_empty());
    assert_eq!(plan["selected_tests"].as_array().unwrap().len(), 1);
    assert!(plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .all(|test| test["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .all(|reason| { reason["changed_file"] != "web/app/deleted.tsx" })));
}

#[test]
fn playwright_json_exports_return_analyzer_reports() {
    let root = fixture("nextjs-coverage", "covered");
    let check = playwright_check_json_impl(json!({ "root": root }).to_string()).unwrap();
    let check: serde_json::Value = serde_json::from_str(&check).unwrap();
    assert!(check["summary"]["totalRoutes"].as_u64().unwrap() > 0);

    let root = fixture("nextjs-coverage", "covered");
    let edges = playwright_edges_json_impl(json!({ "root": root }).to_string()).unwrap();
    let edges: serde_json::Value = serde_json::from_str(&edges).unwrap();
    assert!(!edges["edges"].as_array().unwrap().is_empty());

    let root = fixture("nextjs-coverage", "covered");
    let related = playwright_related_json_impl(
        json!({
            "root": root,
            "files": ["web/app/settings/page.tsx"]
        })
        .to_string(),
    )
    .unwrap();
    let related: serde_json::Value = serde_json::from_str(&related).unwrap();
    assert!(related["tests"]
        .as_array()
        .unwrap()
        .iter()
        .any(|test| test == "tests/e2e/settings.spec.ts"));

    let root = fixture("nextjs-coverage", "covered");
    let tests = playwright_tests_json_impl(json!({ "root": root }).to_string()).unwrap();
    let tests: serde_json::Value = serde_json::from_str(&tests).unwrap();
    assert!(!tests["tests"].as_array().unwrap().is_empty());

    let root = fixture("nextjs-coverage", "covered");
    let error = playwright_related_json_impl(json!({ "root": root }).to_string()).unwrap_err();
    assert!(error
        .reason
        .contains("files must contain at least one file"));
}

#[test]
fn queues_json_returns_project_report() {
    let options = json!({ "root": fixture_root("queue-dashboard/good") }).to_string();
    let output = queues_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(value["jobs"].as_array().unwrap().is_empty());
    assert!(value["diagnostics"].as_array().unwrap().is_empty());
}

#[test]
fn queue_subcommand_json_returns_edges_and_checks() {
    let options = json!({
        "root": fixture("queue-ast-hop", "basic"),
        "files": ["enqueue.ts"],
        "depth": 2
    })
    .to_string();
    let edges: serde_json::Value =
        serde_json::from_str(&queue_edges_json_impl(options).unwrap()).unwrap();
    assert!(!edges.as_array().unwrap().is_empty());

    let options = json!({
        "root": fixture("queue-ast-hop", "basic"),
        "files": ["enqueue.ts"],
        "direction": "deps"
    })
    .to_string();
    let related: serde_json::Value =
        serde_json::from_str(&queue_related_json_impl(options).unwrap()).unwrap();
    assert!(!related.as_array().unwrap().is_empty());

    let options = json!({ "root": fixture_root("queue-dashboard/good") }).to_string();
    let check: serde_json::Value =
        serde_json::from_str(&queue_check_json_impl(options).unwrap()).unwrap();
    assert!(check.as_array().unwrap().is_empty());
}

#[test]
fn server_route_json_returns_reports_edges_and_related() {
    let options = json!({ "root": fixture_root("routes/good") }).to_string();
    let output = server_routes_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(value["summary"]["totalRoutes"].as_u64().unwrap() > 0);

    let options = json!({ "root": fixture_root("routes/good") }).to_string();
    let all_routes: serde_json::Value =
        serde_json::from_str(&server_route_list_json_impl(options).unwrap()).unwrap();
    assert!(all_routes.as_array().unwrap().len() > 1);

    let options = json!({
        "root": fixture_root("routes/good"),
        "files": ["/api/v1/users"]
    })
    .to_string();
    let routes: serde_json::Value =
        serde_json::from_str(&server_route_list_json_impl(options).unwrap()).unwrap();
    assert_eq!(routes.as_array().unwrap().len(), 1);

    let options = json!({
        "root": fixture_root("routes/good"),
        "files": ["backend/api/v1/users.mts"]
    })
    .to_string();
    let edges: serde_json::Value =
        serde_json::from_str(&server_route_edges_json_impl(options).unwrap()).unwrap();
    assert!(!edges.as_array().unwrap().is_empty());

    let options = json!({
        "root": fixture_root("routes/good"),
        "roots": ["backend/api/v1/users.mts"],
        "direction": "dependents"
    })
    .to_string();
    let related: serde_json::Value =
        serde_json::from_str(&server_route_related_json_impl(options).unwrap()).unwrap();
    assert!(related.as_array().is_some());
}

#[test]
fn react_json_functions_return_reports() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/react-traits-analyze/multi-component/fixture"),
    );
    let options = json!({
        "root": root,
        "targets": ["app/components/Mixed.tsx"]
    })
    .to_string();
    let output = react_analyze_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(value
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["name"] == "FetchingComponent"));

    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/react-traits-config/assert-no-fetch/fixture"),
    );
    let options = json!({ "root": root, "assertNoFetch": true }).to_string();
    let output = react_check_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(!value.as_array().unwrap().is_empty());
}

include!("tests_query.rs");

#[test]
fn invalid_options_return_napi_errors() {
    let error = dependencies_json_impl("{}".to_string()).unwrap_err();
    assert!(error
        .reason
        .contains("files must contain at least one file"));

    let error = symbols_json_impl("{}".to_string()).unwrap_err();
    assert!(error
        .reason
        .contains("files must contain at least one file"));

    let error = parse_options::<ProjectOptions>("{").unwrap_err();
    assert!(error.reason.contains("invalid options JSON"));

    let error = parse_options::<ProjectOptions>(r#"{"unknownKey":true}"#).unwrap_err();
    assert!(error.reason.contains("unknown field"));

    let error = parse_options::<TraverseOptions>(r#"{"unknownKey":true}"#).unwrap_err();
    assert!(error.reason.contains("unknown field"));

    let error = parse_options::<SymbolOptions>(r#"{"unknownKey":true}"#).unwrap_err();
    assert!(error.reason.contains("unknown field"));

    let error = queue_related_json_impl(json!({ "files": [] }).to_string()).unwrap_err();
    assert!(error
        .reason
        .contains("files must contain at least one file"));

    let error = server_route_related_json_impl(json!({}).to_string()).unwrap_err();
    assert!(error
        .reason
        .contains("files or roots must contain at least one entry"));

    let error =
        tests_plan_json_impl(json!({ "framework": "unknown", "changedFiles": [] }).to_string())
            .unwrap_err();
    assert!(error.reason.contains("unknown test framework"));

    let error = tests_impact_json_impl(json!({ "entrypoints": [] }).to_string()).unwrap_err();
    assert!(error
        .reason
        .contains("entrypoints is required and must not be empty"));

    let error = tests_why_json_impl(json!({}).to_string()).unwrap_err();
    assert!(error.reason.contains("test is required"));

    let error = tests_comment_markdown_impl(json!({}).to_string()).unwrap_err();
    assert!(error.reason.contains("plan or planJson is required"));

    let error = tests_comment_markdown_impl(json!({ "plan": "does-not-exist.json" }).to_string())
        .unwrap_err();
    assert!(error.reason.contains("Failed to read plan"));
}

#[test]
fn option_parsers_cover_all_supported_values() {
    for relationship in [
        "import",
        "import-static",
        "import-dynamic",
        "import-type",
        "import-require",
        "route-import",
        "workspace",
        "package",
        "test",
        "route",
        "queue",
        "md",
        "ci",
        "http",
        "process",
        "asset",
        "react",
        "dotnet",
        "swift",
        "terraform",
        "all",
    ] {
        parse_relationship(relationship).unwrap();
    }
    assert!(parse_relationship("unknown").is_err());

    for kind in [
        "function",
        "class",
        "const",
        "let",
        "var",
        "type",
        "interface",
        "enum",
        "default",
        "re-export",
    ] {
        parse_export_kind(kind).unwrap();
    }
    assert!(parse_export_kind("unknown").is_err());

    for include in [None, Some("exports"), Some("imports"), Some("both")] {
        parse_include(include).unwrap();
    }
    assert!(parse_include(Some("unknown")).is_err());

    for direction in [None, Some("deps"), Some("dependents"), Some("both")] {
        parse_queue_direction(direction).unwrap();
        parse_server_direction(direction).unwrap();
    }
    assert!(parse_queue_direction(Some("unknown")).is_err());
    assert!(parse_server_direction(Some("unknown")).is_err());

    assert!(resolve_project_root(None).unwrap().is_absolute());
    assert_eq!(
        project_roots(&ProjectOptions {
            files: vec!["file.ts".to_string()],
            ..ProjectOptions::default()
        }),
        vec!["file.ts".to_string()]
    );
}

#[test]
fn relationship_parser_accepts_conservative_route_import_edges() {
    assert_eq!(
        parse_relationship("route-import").unwrap(),
        crate::codebase::dependencies::RelationshipArg::RouteImport
    );
}

include!("tests_impact.rs");
include!("tests_impact_fallback.rs");
include!("tests_queries.rs");
include!("tests_playwright.rs");

mod check;
mod ci;
mod react_usages;
mod tests_entrypoints;
mod tests_sample_when_limited;
