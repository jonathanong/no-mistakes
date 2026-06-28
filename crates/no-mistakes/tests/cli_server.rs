use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn server_fixture(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/server-ast-routes")
            .join(name)
            .join("fixture"),
    )
}

fn run(args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("no-mistakes should run")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf8")
}

#[test]
fn server_routes_json_lists_routes() {
    let root = server_fixture("express");
    let output = run(&[
        "server",
        "--root",
        root.to_str().unwrap(),
        "--json",
        "routes",
    ]);

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(json
        .as_array()
        .unwrap()
        .iter()
        .any(|r| { r["route"].as_str().unwrap_or("").contains("/api/v1/users") }));
    let search = json
        .as_array()
        .unwrap()
        .iter()
        .find(|route| route["route"] == "/api/v1/search")
        .expect("search route should be extracted");
    assert_eq!(search["queryParams"], serde_json::json!(["page", "term"]));
    let shapes = json
        .as_array()
        .unwrap()
        .iter()
        .find(|route| route["route"] == "/api/v1/query-shapes")
        .expect("query shape fixture route should be extracted");
    let params = shapes["queryParams"].as_array().unwrap();
    for param in [
        "array",
        "arrowBody",
        "call",
        "calls",
        "conditional",
        "elseBranch",
        "finallyBlock",
        "first",
        "forBody",
        "functionBody",
        "functionExpressionBody",
        "object",
        "switchOn",
        "tryBlock",
        "url",
        "whileLoop",
    ] {
        assert!(
            params.iter().any(|value| value == param),
            "missing query param {param}: {params:?}"
        );
    }
    assert!(
        !params
            .iter()
            .any(|value| value == "alias" || value == "ignored"),
        "non-query aliases should not be extracted: {params:?}"
    );
}

#[test]
fn server_contracts_reports_client_query_mismatches() {
    let root = server_fixture("express");
    let output = run(&[
        "server",
        "--root",
        root.to_str().unwrap(),
        "--json",
        "contracts",
    ]);

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(json["routes"].as_array().unwrap().iter().any(|route| {
        route["route"] == "/api/v1/search"
            && route["queryParams"] == serde_json::json!(["page", "term"])
    }));
    assert!(json["mismatches"]
        .as_array()
        .unwrap()
        .iter()
        .any(|mismatch| {
            mismatch["matchedRoute"] == "/api/v1/search"
                && mismatch["missingParams"] == serde_json::json!(["unused"])
        }));
}

#[test]
fn server_contracts_supports_all_render_formats() {
    let root = server_fixture("express");
    for format in ["human", "md", "paths", "yml"] {
        let output = run(&[
            "server",
            "--root",
            root.to_str().unwrap(),
            "--format",
            format,
            "contracts",
        ]);

        assert!(
            output.status.success(),
            "{format} stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = stdout(&output);
        assert!(!stdout.is_empty(), "{format} should render contracts");
    }
}

#[test]
fn server_edges_human_shows_edges() {
    let root = server_fixture("express");
    let output = run(&["server", "--root", root.to_str().unwrap(), "edges"]);

    assert!(output.status.success());
    assert!(!stdout(&output).is_empty());
}

#[test]
fn server_related_json_shows_edges() {
    let root = server_fixture("express");
    let output = run(&[
        "server",
        "--root",
        root.to_str().unwrap(),
        "--json",
        "related",
        "backend/api/users.ts",
    ]);

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(json.as_array().is_some());
}

#[test]
fn server_routes_human_format() {
    let root = server_fixture("express");
    let output = run(&["server", "--root", root.to_str().unwrap(), "routes"]);

    assert!(output.status.success());
    assert!(stdout(&output).contains("/api/v1/users"));
}

#[test]
fn server_routes_with_file_filter() {
    let root = server_fixture("express");
    let output = run(&[
        "server",
        "--root",
        root.to_str().unwrap(),
        "routes",
        "backend/api/users.ts",
    ]);

    assert!(output.status.success());
    assert!(stdout(&output).contains("/api/v1/users"));
}

#[test]
fn server_edges_json_format() {
    let root = server_fixture("express");
    let output = run(&[
        "server",
        "--root",
        root.to_str().unwrap(),
        "--json",
        "edges",
    ]);

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(json.as_array().is_some());
}

#[test]
fn server_edges_with_root_filter() {
    let root = server_fixture("express");
    let output = run(&[
        "server",
        "--root",
        root.to_str().unwrap(),
        "edges",
        "backend/api/users.ts",
    ]);

    assert!(output.status.success());
}

#[test]
fn server_related_human_format() {
    let root = server_fixture("express");
    let output = run(&[
        "server",
        "--root",
        root.to_str().unwrap(),
        "related",
        "backend/api/users.ts",
    ]);

    assert!(output.status.success());
}

#[test]
fn server_routes_json_with_file_filter() {
    let root = server_fixture("express");
    let output = run(&[
        "server",
        "--root",
        root.to_str().unwrap(),
        "--json",
        "routes",
        "backend/api/users.ts",
    ]);

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(!json.as_array().unwrap().is_empty());
}

#[test]
fn server_edges_json_with_root_filter() {
    let root = server_fixture("express");
    let output = run(&[
        "server",
        "--root",
        root.to_str().unwrap(),
        "--json",
        "edges",
        "backend/api/users.ts",
    ]);

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(json.as_array().is_some());
}

#[test]
fn server_related_direction_deps() {
    let root = server_fixture("express");
    let output = run(&[
        "server",
        "--root",
        root.to_str().unwrap(),
        "related",
        "backend/api/users.ts",
        "--direction",
        "deps",
    ]);

    assert!(output.status.success());
}

#[test]
fn server_related_direction_dependents() {
    let root = server_fixture("express");
    let output = run(&[
        "server",
        "--root",
        root.to_str().unwrap(),
        "related",
        "backend/api/users.ts",
        "--direction",
        "dependents",
    ]);

    assert!(output.status.success());
}

#[test]
fn server_relative_root_is_resolved() {
    let output = Command::new(bin())
        .current_dir(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .unwrap(),
        )
        .args([
            "server",
            "--root",
            "test-cases/server-ast-routes/express/fixture",
            "routes",
        ])
        .output()
        .expect("no-mistakes should run");

    assert!(output.status.success());
}
