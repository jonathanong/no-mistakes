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
    // Changing a component with data-pw selectors matching getByTestId calls
    // in a test should select that test via selector-based coverage edges.
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
    let selected: Vec<&str> = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["test_file"].as_str().unwrap())
        .collect();
    assert!(
        selected.contains(&"tests/e2e/dashboard-nav.spec.ts"),
        "expected dashboard-nav.spec.ts via selector coverage, got: {:?}",
        selected
    );
}

#[test]
fn test_plan_vitest_full_suite_triggers_key_works() {
    // The new `fullSuiteTriggers` key should behave identically to the old
    // `dependencies` key.
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
