mod common;

use common::{fixture, run, stdout};

#[test]
fn test_plan_vitest_project_dependency_fallback_honors_file_limit() {
    let root = fixture("test-plan-config");
    let output = run(&[
        "test",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "web/app/page.tsx",
        "--limit-files",
        "1",
        "--json",
    ]);

    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], true);
    assert!(plan["fallback_reason"]
        .as_str()
        .unwrap()
        .contains("web project dependency changed"));
    assert_eq!(plan["selected_tests"].as_array().unwrap().len(), 1);
    assert_eq!(plan["groups"][0]["type"], "dependencies");
    assert_eq!(plan["groups"][0]["selected"].as_array().unwrap().len(), 1);
    assert_eq!(plan["groups"][0]["limit"], 1);
    assert_eq!(plan["groups"][0]["remaining"], 1);
}

#[test]
fn test_plan_vitest_project_dependency_fallback_honors_percent_limit() {
    let root = fixture("test-plan-config");
    let output = run(&[
        "test",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "web/app/page.tsx",
        "--limit-percent",
        "50",
        "--json",
    ]);

    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], true);
    assert_eq!(plan["selected_tests"].as_array().unwrap().len(), 1);
    assert_eq!(plan["groups"][0]["limit"], 1);
    assert_eq!(plan["groups"][0]["remaining"], 1);
}

#[test]
fn test_plan_vitest_all_environment_honors_limit_override() {
    let root = fixture("test-plan-config");
    let output = run(&[
        "test",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "source.ts",
        "--environment",
        "all",
        "--limit-files",
        "1",
        "--json",
    ]);

    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], true);
    assert_eq!(plan["groups"][0]["type"], "all");
    assert_eq!(plan["selected_tests"].as_array().unwrap().len(), 1);
    assert_eq!(plan["groups"][0]["selected"].as_array().unwrap().len(), 1);
    assert_eq!(plan["groups"][0]["limit"], 1);
    assert_eq!(plan["groups"][0]["remaining"], 1);
}

#[test]
fn test_plan_playwright_dependency_can_ignore_changed_vitest_tests() {
    let root = fixture("test-plan-ignore-changed-tests");
    let output = run(&[
        "test",
        "plan",
        "playwright",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "cloudflare-worker/src/__tests__/index.routing.backend-cache.mock.test.mts",
        "--changed-file",
        ".github/workflows/static-code-analysis.test.mts",
        "--json",
    ]);

    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], false);
    assert!(plan["fallback_reason"].is_null());
    assert!(plan["selected_tests"].as_array().unwrap().is_empty());
}

#[test]
fn test_plan_playwright_dependency_still_triggers_for_non_test_project_file() {
    let root = fixture("test-plan-ignore-changed-tests");
    let output = run(&[
        "test",
        "plan",
        "playwright",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "cloudflare-worker/src/index.mts",
        "--json",
    ]);

    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], true);
    assert!(plan["fallback_reason"]
        .as_str()
        .unwrap()
        .contains("cloudflare-worker project dependency changed"));
    assert_eq!(plan["selected_tests"].as_array().unwrap().len(), 1);
    assert_eq!(plan["groups"][0]["type"], "dependencies");
    assert_eq!(plan["groups"][0]["limit"], 1);
    assert_eq!(plan["groups"][0]["remaining"], 1);
}
