use crate::server_routes::{analyze_project, prepare_analysis};
use std::path::PathBuf;

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
}

#[test]
fn go_http_filter_excludes_route_file() {
    let report = analyze_project(&lang_fixture("go-http"), None, &["computed.go".into()]).unwrap();
    assert!(report.routes.iter().all(|route| route.route != "/health"));
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
