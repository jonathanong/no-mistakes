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
