use crate::codebase::lang_frontends::LangFileFacts;
use crate::server_routes::{analyze_project, prepare_analysis};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn lang_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/lang-frontends")
        .join(name)
}

fn ts_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/server-ast-routes")
        .join(name)
        .join("fixture")
}

fn codebase_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis")
        .join(name)
        .join("fixture")
}

#[test]
fn missing_config_does_not_add_language_routes() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/analysis-dataset/source-store");
    let report = analyze_project(&root, None, &[]).unwrap();
    assert!(report.routes.is_empty());
}

#[test]
fn language_packages_do_not_change_typescript_routes() {
    let root = ts_fixture("express");
    let off = analyze_project(&root, None, &[]).unwrap();
    let prepared = prepare_analysis(&root, None).unwrap();
    let on = crate::server_routes::analyze_project_with_prepared(&prepared, &[]).unwrap();
    assert_eq!(off.routes, on.routes);
    assert_eq!(off.edges, on.edges);
}

#[test]
fn go_http_fixture_lists_literal_routes() {
    let report = analyze_project(&lang_fixture("go-http"), None, &[]).unwrap();
    assert!(report
        .routes
        .iter()
        .any(|route| route.route == "/health" && route.file.ends_with("routes.go")));
    assert!(report
        .edges
        .iter()
        .any(|edge| edge.from.ends_with("routes.go") && edge.to == "/health"));
    assert!(report
        .routes
        .iter()
        .all(|route| !route.file.ends_with("computed.go")));
    assert!(report
        .routes
        .iter()
        .all(|route| route.route != "/from-test"));
}

#[test]
fn go_http_filter_excludes_route_file() {
    let report = analyze_project(&lang_fixture("go-http"), None, &["computed.go".into()]).unwrap();
    assert!(report.routes.iter().all(|route| route.route != "/health"));
}

#[test]
fn server_route_globs_exclude_language_route_files() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/server-queues-lang/go-http-glob-exclude");
    let report = analyze_project(&root, None, &[]).unwrap();
    assert!(report.routes.iter().all(|route| route.route != "/health"));
}

#[test]
fn merge_file_routes_skips_files_without_handlers() {
    let root = Path::new("/repo");
    let file = LangFileFacts {
        path: root.join("app.go"),
        ..Default::default()
    };
    let mut facts = HashMap::new();
    super::merge_file_routes(root, &file, &mut facts, None, None, None);
    assert!(facts.is_empty());
}

#[test]
fn flask_fastapi_fixture_lists_literal_routes() {
    let report = analyze_project(&lang_fixture("python-flask-fastapi"), None, &[]).unwrap();
    assert!(report
        .routes
        .iter()
        .any(|route| route.file.ends_with("flask_app.py")));
    assert!(report
        .routes
        .iter()
        .any(|route| route.file.ends_with("fastapi_app.py")));
    assert!(report
        .routes
        .iter()
        .all(|route| !route.file.ends_with("computed.py")));
}

#[test]
fn rust_http_fixture_lists_literal_routes() {
    let report = analyze_project(&lang_fixture("rust-http"), None, &[]).unwrap();
    assert!(report
        .routes
        .iter()
        .any(|route| route.route == "/users" && route.file.ends_with("routes.rs")));
    assert!(report
        .routes
        .iter()
        .any(|route| route.route == "/health" && route.file.ends_with("handlers.rs")));
    assert!(report
        .routes
        .iter()
        .all(|route| !route.file.ends_with("computed.rs")));
}

#[test]
fn aspnet_fixture_lists_literal_routes() {
    let report = analyze_project(&codebase_fixture("dotnet-aspnet-routes"), None, &[]).unwrap();
    assert!(report
        .routes
        .iter()
        .any(|route| route.route == "/users" && route.file.ends_with("Program.cs")));
    assert!(report
        .routes
        .iter()
        .any(|route| route.route == "/orders" && route.file.ends_with("UsersController.cs")));
    assert!(report
        .routes
        .iter()
        .all(|route| !route.file.ends_with("Computed.cs")));
}

#[test]
fn spring_fixture_lists_literal_routes() {
    let report = analyze_project(&lang_fixture("java-spring"), None, &[]).unwrap();
    assert!(report
        .routes
        .iter()
        .any(|route| { route.route == "/api/users" && route.file.ends_with("Users.java") }));
    assert!(report
        .routes
        .iter()
        .all(|route| !route.file.ends_with("Computed.java")));
}

#[test]
fn kotlin_spring_fixture_lists_literal_routes() {
    let report = analyze_project(&lang_fixture("kotlin-spring"), None, &[]).unwrap();
    assert!(report
        .routes
        .iter()
        .any(|route| { route.route == "/api/users" && route.file.ends_with("Users.kt") }));
    assert!(report
        .routes
        .iter()
        .all(|route| !route.file.ends_with("Computed.kt")));
}

#[test]
fn phoenix_fixture_lists_literal_routes() {
    let report = analyze_project(&lang_fixture("phoenix-routes"), None, &[]).unwrap();
    assert!(report
        .routes
        .iter()
        .any(|route| { route.route == "/users" && route.file.ends_with("router.ex") }));
    assert!(report
        .routes
        .iter()
        .all(|route| !route.file.ends_with("computed.ex")));
}
