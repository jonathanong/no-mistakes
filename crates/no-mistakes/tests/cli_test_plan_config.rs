use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase-analysis")
            .join(name),
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

fn only_reason_via(plan: &serde_json::Value, test_file: &str) -> Vec<String> {
    let selected = plan["selected_tests"].as_array().unwrap();
    let test = selected
        .iter()
        .find(|test| test["test_file"] == test_file)
        .unwrap();
    let reasons = test["reasons"].as_array().unwrap();
    assert_eq!(reasons.len(), 1);
    reasons[0]["via"]
        .as_array()
        .unwrap()
        .iter()
        .map(|kind| kind.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn test_plan_vitest_applies_configured_groups_and_limits() {
    let root = fixture("test-plan-config");
    let output = run(&[
        "test",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "source.ts",
        "--json",
    ]);

    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], false);
    let selected = plan["selected_tests"].as_array().unwrap();
    assert_eq!(selected.len(), 2);
    let names: Vec<&str> = selected
        .iter()
        .map(|test| test["test_file"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"source.test.mts"));
    assert!(names.contains(&"other.test.mts"));
    assert_eq!(plan["groups"][0]["type"], "direct");
    assert_eq!(plan["groups"][1]["type"], "dependencies");
    assert_eq!(plan["groups"][2]["type"], "sample");
}

#[test]
fn test_plan_vitest_direct_group_is_mutually_exclusive() {
    let root = fixture("test-plan-config");
    let output = run(&[
        "test",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "source.test.mts",
        "--json",
    ]);

    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["selected_tests"].as_array().unwrap().len(), 2);
    assert_eq!(plan["groups"][0]["selected"][0], "source.test.mts");
    assert_eq!(plan["groups"][1]["selected"].as_array().unwrap().len(), 0);
}

#[test]
fn test_plan_vitest_project_dependency_triggers_all_tests() {
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
    assert_eq!(plan["fallback_triggered"], true);
    assert!(plan["fallback_reason"]
        .as_str()
        .unwrap()
        .contains("web project dependency changed"));
    assert_eq!(plan["selected_tests"].as_array().unwrap().len(), 2);
}

#[test]
fn test_plan_vitest_project_dependency_patterns_are_project_relative() {
    let root = fixture("test-plan-config");
    let output = run(&[
        "test",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "patterns/next.config.mjs",
        "--json",
    ]);

    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], true);
    assert!(plan["fallback_reason"]
        .as_str()
        .unwrap()
        .contains("web-patterns project dependency changed"));
    assert_eq!(plan["selected_tests"].as_array().unwrap().len(), 2);
}

#[test]
fn test_plan_vitest_project_dependency_include_is_project_relative() {
    let root = fixture("test-plan-config");
    let output = run(&[
        "test",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "config/next.config.mjs",
        "--json",
    ]);

    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], true);
    assert!(plan["fallback_reason"]
        .as_str()
        .unwrap()
        .contains("config-include project dependency changed"));
    assert_eq!(plan["selected_tests"].as_array().unwrap().len(), 2);
}

#[test]
fn test_plan_vitest_all_environment_runs_all_tests() {
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
        "--json",
    ]);

    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], true);
    assert!(plan["fallback_reason"]
        .as_str()
        .unwrap()
        .contains("environment `all` runs all tests"));
    assert_eq!(plan["groups"][0]["type"], "all");
    assert_eq!(plan["selected_tests"].as_array().unwrap().len(), 2);
}

#[test]
fn test_plan_vitest_limit_overrides_configured_limit() {
    let root = fixture("test-plan-config");
    let output = run(&[
        "test",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "source.ts",
        "--limit-files",
        "1",
        "--json",
    ]);

    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["selected_tests"].as_array().unwrap().len(), 1);
    assert_eq!(plan["groups"][1]["selected"][0], "source.test.mts");
    assert_eq!(plan["groups"][2]["limit"], 0);
    assert_eq!(plan["groups"][2]["remaining"], 1);
}

#[test]
fn test_plan_vitest_rejects_coverage_group() {
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
        "coverage-only",
        "--json",
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr.clone()).unwrap();
    assert!(stderr.contains("vitest test plans do not support the coverage group"));
}

#[test]
fn test_plan_playwright_uses_coverage_group() {
    let root = fixture("playwright-impact-routing");
    let output = run(&[
        "test",
        "plan",
        "playwright",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "web/components/UserCard.tsx",
        "--json",
    ]);

    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], false);
    assert_eq!(plan["selected_tests"].as_array().unwrap().len(), 1);
    assert_eq!(plan["groups"][0]["type"], "direct");
    assert_eq!(plan["groups"][1]["type"], "coverage");
    assert_eq!(plan["groups"][1]["selected"][0], "tests/e2e/routes.spec.ts");
    assert_eq!(
        only_reason_via(&plan, "tests/e2e/routes.spec.ts"),
        vec!["dependency", "route"]
    );
}
