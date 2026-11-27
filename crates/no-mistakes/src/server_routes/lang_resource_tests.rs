use super::lang_fixture;
use crate::server_routes::analyze_project;

#[test]
fn laravel_fixture_lists_literal_resource_and_api_resource_routes() {
    let report = analyze_project(&lang_fixture("php-laravel"), None, &[]).unwrap();
    assert!(report
        .routes
        .iter()
        .any(|route| { route.route == "/users" && route.file.ends_with("web.php") }));
    assert!(report
        .routes
        .iter()
        .any(|route| { route.route == "/accounts" && route.file.ends_with("web.php") }));
    assert!(report
        .routes
        .iter()
        .any(|route| { route.route == "/accounts/*" && route.file.ends_with("web.php") }));
    assert!(web_route(&report, "/hidden"));
    assert!(!web_route(&report, "/hidden/*"));
    assert!(!web_route(&report, "/hidden/create"));
    assert!(web_route(&report, "/articles/*"));
    assert!(!web_route(&report, "/articles"));
    assert!(web_route(&report, "/labels"));
    assert!(!web_route(&report, "/labels/*"));
    assert!(web_route(&report, "/photos/*/comments"));
    assert!(web_route(&report, "/photos/*/comments/create"));
    assert!(web_route(&report, "/photos/*/comments/*"));
    assert!(web_route(&report, "/photos/*/comments/*/edit"));
    assert!(!web_route(&report, "/photos"));
    assert!(web_route(&report, "/named-users"));
    assert!(web_route(&report, "/named-users/*"));
    assert!(!web_route(&report, "/named-users/create"));
    assert!(web_route(&report, "/named-accounts"));
    assert!(web_route(&report, "/named-accounts/*"));
    assert!(web_route(&report, "/bulk"));
    assert!(web_route(&report, "/bulk/*"));
    assert!(!web_route(&report, "/bulk/create"));
    assert!(web_route(&report, "/filtered"));
    assert!(!web_route(&report, "/filtered/*"));
    assert!(web_route(&report, "/excluded"));
    assert!(web_route(&report, "/excluded/*"));
    assert!(report.routes.iter().all(|route| {
        !route.route.contains('$') && route.route != "/files" && !route.file.ends_with("files")
    }));
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
        .any(|route| { route.route == "/api/items" && route.file.ends_with("router.ex") }));
    assert!(report
        .routes
        .iter()
        .any(|route| { route.route == "/g/h" && route.file.ends_with("router.ex") }));
    assert!(report
        .routes
        .iter()
        .any(|route| { route.route == "/z" && route.file.ends_with("router.ex") }));
    assert!(report
        .routes
        .iter()
        .all(|route| { !route.route.contains("#{") && route.route != "/api/after-comment-end" }));
    assert!(report
        .routes
        .iter()
        .all(|route| !route.file.ends_with("computed.ex")));
    assert!(report
        .routes
        .iter()
        .any(|route| { route.route == "/accounts/new" && route.file.ends_with("router.ex") }));
    assert!(report
        .routes
        .iter()
        .any(|route| { route.route == "/accounts/*" && route.file.ends_with("router.ex") }));
    assert!(report
        .routes
        .iter()
        .any(|route| { route.route == "/accounts/*/edit" && route.file.ends_with("router.ex") }));
    assert!(report
        .routes
        .iter()
        .any(|route| { route.route == "/v1/members" && route.file.ends_with("router.ex") }));
    assert!(report
        .routes
        .iter()
        .any(|route| { route.route == "/v1/members/*/edit" && route.file.ends_with("router.ex") }));
}

fn web_route(report: &crate::server_routes::ProjectReport, path: &str) -> bool {
    report
        .routes
        .iter()
        .any(|route| route.route == path && route.file.ends_with("web.php"))
}

#[test]
fn laravel_and_phoenix_route_json_matches_async_napi() {
    assert_route_json_parity(&lang_fixture("php-laravel"), "/hidden");
    assert_route_json_parity(&lang_fixture("phoenix-routes"), "/accounts/new");
    let root = lang_fixture("php-laravel");
    let options = serde_json::json!({
        "root": root.display().to_string(),
        "files": ["app/Http/Controllers/PhotoCommentController.php"],
        "relationships": ["route"]
    });
    let output =
        crate::napi_api::dependents_json_impl(crate::napi_api::options::test_json_arg(options))
            .unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(value["files"].as_array().unwrap().iter().any(|file| {
        file["path"]
            .as_str()
            .is_some_and(|path| path.ends_with("routes/web.php"))
    }));
}

fn assert_route_json_parity(root: &std::path::Path, needle: &str) {
    let report = analyze_project(root, None, &[]).unwrap();
    let cli: serde_json::Value =
        serde_json::from_str(&crate::cli::json_string(&report.routes)).unwrap();
    let options = crate::napi_api::options::test_json_arg(serde_json::json!({
        "root": root.display().to_string()
    }));
    let list: serde_json::Value = serde_json::from_str(
        &crate::napi_api::server_route_list_json_impl(options.clone()).unwrap(),
    )
    .unwrap();
    let full: serde_json::Value =
        serde_json::from_str(&crate::napi_api::server_routes_json_impl(options).unwrap()).unwrap();
    assert_eq!(list, cli);
    assert_eq!(full["routes"], cli);
    assert!(list
        .as_array()
        .unwrap()
        .iter()
        .any(|route| route["route"] == needle));
}
