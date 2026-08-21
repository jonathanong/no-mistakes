use super::path::{route_from_app_root, route_from_routes_rel};
use crate::server_routes::{analyze_project, prepare_analysis};
use std::path::PathBuf;

fn remix_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/server-ast-routes/remix/fixture")
}

fn ts_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/server-ast-routes")
        .join(name)
        .join("fixture")
}

#[test]
fn maps_flat_and_folder_route_files() {
    assert_eq!(
        route_from_routes_rel("users.$id.tsx").as_deref(),
        Some("/users/:id")
    );
    assert_eq!(
        route_from_routes_rel("users/$id.tsx").as_deref(),
        Some("/users/:id")
    );
    assert_eq!(route_from_routes_rel("_index.tsx").as_deref(), Some("/"));
    assert_eq!(
        route_from_routes_rel("_auth.login.tsx").as_deref(),
        Some("/login")
    );
    assert_eq!(route_from_routes_rel("$.tsx").as_deref(), Some("/*"));
    assert_eq!(
        route_from_routes_rel("concerts_.mine.tsx").as_deref(),
        Some("/concerts/mine")
    );
    assert_eq!(
        route_from_routes_rel("users._index.tsx").as_deref(),
        Some("/users")
    );
    assert_eq!(
        route_from_routes_rel("users/index.tsx").as_deref(),
        Some("/users")
    );
    assert_eq!(route_from_routes_rel("users.$id.server.ts"), None);
    assert_eq!(route_from_app_root("app/root.tsx").as_deref(), Some("/"));
    assert_eq!(route_from_app_root("app/routes/users.tsx"), None);
}

#[test]
fn remix_fixture_lists_file_based_routes() {
    let report = analyze_project(&remix_fixture(), None, &[]).unwrap();
    let routes: Vec<_> = report
        .routes
        .iter()
        .map(|route| {
            (
                route.file.as_str(),
                route.route.as_str(),
                route.raw_path.as_str(),
                route.framework,
            )
        })
        .collect();
    assert!(
        routes.iter().any(|(file, route, raw, framework)| {
            file.ends_with("app/routes/users.$id.tsx")
                && *route == "/users/*"
                && *raw == "/users/:id"
                && *framework == crate::server_routes::Framework::Remix
        }),
        "missing users.$id remix route in {routes:?}"
    );
    assert!(routes
        .iter()
        .any(|(file, route, _, _)| file.ends_with("app/routes/_index.tsx") && *route == "/"));
    assert!(routes.iter().any(|(file, route, _, _)| {
        file.ends_with("app/routes/_auth.login.tsx") && *route == "/login"
    }));
    assert!(routes
        .iter()
        .any(|(file, route, _, _)| file.ends_with("app/routes/$.tsx") && *route == "/*"));
    assert!(routes.iter().any(|(file, route, _, _)| {
        file.ends_with("app/routes/concerts_.mine.tsx") && *route == "/concerts/mine"
    }));
    assert!(routes
        .iter()
        .any(|(file, route, _, _)| file.ends_with("app/root.tsx") && *route == "/"));
    assert!(routes
        .iter()
        .all(|(file, _, _, _)| !file.ends_with("users.$id.server.ts")));
    assert!(routes
        .iter()
        .all(|(file, _, _, _)| !file.contains("not-a-route")));
}

#[test]
fn remix_does_not_change_express_routes() {
    let root = ts_fixture("express");
    let off = analyze_project(&root, None, &[]).unwrap();
    let prepared = prepare_analysis(&root, None).unwrap();
    let on = crate::server_routes::analyze_project_with_prepared(&prepared, &[]).unwrap();
    assert_eq!(off.routes, on.routes);
    assert_eq!(off.edges, on.edges);
}

#[test]
fn missing_remix_project_does_not_index_route_files() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/server-ast-routes/remix-unconfigured/fixture");
    let report = analyze_project(&root, None, &[]).unwrap();
    assert!(report.routes.is_empty());
}

#[test]
fn remix_cli_filter_keeps_matching_route_files() {
    let report = analyze_project(&remix_fixture(), None, &["**/_index.tsx".into()]).unwrap();
    assert!(report
        .routes
        .iter()
        .any(|route| route.file.ends_with("app/routes/_index.tsx")));
    assert!(report
        .routes
        .iter()
        .all(|route| route.file.ends_with("app/routes/_index.tsx")));
}

#[test]
fn inferred_remix_root_without_route_modules_lists_nothing() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/config-v2/remix-inferred-root/fixture");
    let report = analyze_project(&root, None, &[]).unwrap();
    assert!(report.routes.is_empty());
}
