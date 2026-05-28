mod common;

use common::{fixture, run, stdout};

#[test]
fn test_plan_playwright_coverage_route_group_component() {
    // Changing a component imported by a route-group layout should select
    // tests that visit routes under that group via the coverage group.
    let root = fixture("playwright-coverage-route-group");
    let output = run(&[
        "test",
        "plan",
        "playwright",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "web/components/dashboard-nav.tsx",
        "--environment",
        "prePush",
        "--json",
    ]);
    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], false);
    let selected: Vec<&str> = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["test_file"].as_str().unwrap())
        .collect();
    assert!(
        selected.contains(&"tests/e2e/dashboard-nav.spec.ts"),
        "expected dashboard-nav.spec.ts in selected, got: {:?}",
        selected
    );
    // Verify it came through the coverage group.
    let coverage_group = plan["groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["type"] == "coverage")
        .unwrap();
    assert!(
        coverage_group["selected"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str() == Some("tests/e2e/dashboard-nav.spec.ts")),
        "expected dashboard-nav.spec.ts in coverage group selected"
    );
}

#[test]
fn test_plan_playwright_coverage_route_group_page() {
    // Changing a page inside a route-group directory should select tests
    // visiting that route.
    let root = fixture("playwright-coverage-route-group");
    let output = run(&[
        "test",
        "plan",
        "playwright",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "web/app/(dashboard)/account/settings/page.tsx",
        "--environment",
        "prePush",
        "--json",
    ]);
    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], false);
    let selected: Vec<&str> = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["test_file"].as_str().unwrap())
        .collect();
    assert!(
        selected.contains(&"tests/e2e/dashboard-nav.spec.ts"),
        "expected dashboard-nav.spec.ts in selected, got: {:?}",
        selected
    );
}

#[test]
fn test_plan_playwright_coverage_selector_edges() {
    // Changing search-bar.tsx — a component with data-pw selectors that is NOT
    // imported by any layout/page — should select search-bar.spec.ts exclusively
    // via selector-based coverage edges (not layout or route-test edges).
    let root = fixture("playwright-coverage-route-group");
    let output = run(&[
        "test",
        "plan",
        "playwright",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "web/components/search-bar.tsx",
        "--environment",
        "prePush",
        "--json",
    ]);
    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let selected: Vec<&str> = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["test_file"].as_str().unwrap())
        .collect();
    assert!(
        selected.contains(&"tests/e2e/search-bar.spec.ts"),
        "expected search-bar.spec.ts via selector coverage, got: {:?}",
        selected
    );
    // Verify the selection reason is exclusively selector-based.
    let entry = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["test_file"].as_str() == Some("tests/e2e/search-bar.spec.ts"))
        .unwrap();
    let via: Vec<&str> = entry["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|r| r["via"].as_array().unwrap())
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(
        via,
        vec!["selector"],
        "search-bar.spec.ts must be selected via selector edge only, got: {:?}",
        via
    );
}

fn via_labels(plan: &serde_json::Value, test_file: &str) -> Vec<String> {
    let entry = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["test_file"].as_str() == Some(test_file))
        .unwrap_or_else(|| panic!("{test_file} not in selected_tests: {plan:#}"));
    entry["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|r| r["via"].as_array().unwrap())
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn test_plan_playwright_coverage_layout_wrapper_component() {
    // Issue #280: changing a layout-wrapper component with NO data-pw of its own
    // must select playwright specs that navigate to routes under the wrapping
    // layout, via the route → layout edge chain (not via selectors).
    let root = fixture("playwright-coverage-route-layout");
    let output = run(&[
        "test",
        "plan",
        "playwright",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "web/components/page-with-aside.tsx",
        "--environment",
        "prePush",
        "--json",
    ]);
    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], false);
    let via = via_labels(&plan, "tests/e2e/feed.spec.ts");
    assert!(
        via.iter().any(|v| v == "layout"),
        "expected a 'layout' edge in via for feed.spec.ts, got: {via:?}"
    );
    assert!(
        !via.iter().any(|v| v == "selector"),
        "feed.spec.ts must not be selected via selector edges (PageWithAside has no data-pw), got via: {via:?}"
    );
}

#[test]
fn test_plan_playwright_coverage_layout_file_directly() {
    // Issue #280: changing the route-group layout file itself must select
    // playwright specs that visit routes under that layout.
    let root = fixture("playwright-coverage-route-layout");
    let output = run(&[
        "test",
        "plan",
        "playwright",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "web/app/(feed)/layout.tsx",
        "--environment",
        "prePush",
        "--json",
    ]);
    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], false);
    let via = via_labels(&plan, "tests/e2e/feed.spec.ts");
    assert!(
        via.iter().any(|v| v == "layout"),
        "expected 'layout' in via for feed.spec.ts after changing (feed)/layout.tsx, got: {via:?}"
    );
    assert!(
        !via.iter().any(|v| v == "selector"),
        "feed.spec.ts must not be selected via selector edges, got via: {via:?}"
    );
}

#[test]
fn test_plan_playwright_coverage_navigate_to_helper() {
    // Issue #280: a spec that uses a custom `navigateTo(page, '/url')` helper
    // (configured via navigationHelpers) must also propagate layout-wrapper
    // changes — not only specs using bare `page.goto()`.
    let root = fixture("playwright-coverage-route-layout");
    let output = run(&[
        "test",
        "plan",
        "playwright",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "web/components/page-with-aside.tsx",
        "--environment",
        "prePush",
        "--json",
    ]);
    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], false);
    let via = via_labels(&plan, "tests/e2e/feed-nav.spec.ts");
    assert!(
        via.iter().any(|v| v == "layout"),
        "expected 'layout' in via for feed-nav.spec.ts (uses navigateTo helper), got: {via:?}"
    );
    assert!(
        !via.iter().any(|v| v == "selector"),
        "feed-nav.spec.ts must not be selected via selector edges, got via: {via:?}"
    );
}

#[test]
fn test_plan_vitest_deprecated_dependencies_key_still_triggers() {
    // The fixture uses the deprecated `dependencies` key; backward compat
    // should preserve the trigger behaviour identical to `fullSuiteTriggers`.
    let root = fixture("test-plan-config");
    let output = run(&[
        "test",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "web/app/page.tsx",
        "--json",
    ]);
    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    // The existing fixture uses the deprecated `dependencies` key — verify
    // backward compat: the trigger still fires.
    assert_eq!(plan["fallback_triggered"], true);
    assert!(plan["fallback_reason"]
        .as_str()
        .unwrap()
        .contains("web project dependency changed"));
}
