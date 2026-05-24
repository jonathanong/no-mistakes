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
    assert!(plan["groups"][0]["limit"].is_null());
}

#[test]
fn test_plan_vitest_ignores_deleted_project_dependency_paths() {
    let root = fixture("test-plan-config");
    let output = run(&[
        "test",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "web/app/deleted.tsx",
        "--changed-file",
        "source.ts",
        "--json",
    ]);

    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], false);
    assert!(plan["fallback_reason"].is_null());
    assert!(plan["warnings"].as_array().unwrap().is_empty());
    assert!(plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .all(|test| test["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .all(|reason| { reason["changed_file"] != "web/app/deleted.tsx" })));
}

#[test]
fn test_plan_vitest_global_config_change_fallbacks_by_default() {
    let root = fixture("test-plan-config");
    let output = run(&[
        "test",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        ".no-mistakes.yml",
        "--json",
    ]);

    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], true);
    assert!(plan["fallback_reason"]
        .as_str()
        .unwrap()
        .contains("Global configuration file changed: .no-mistakes.yml"));
    assert_eq!(plan["selected_tests"].as_array().unwrap().len(), 2);
    assert_eq!(plan["groups"][0]["type"], "global");
}

#[test]
fn test_plan_vitest_environment_can_disable_global_config_fallback() {
    let root = fixture("test-plan-config");
    let output = run(&[
        "test",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        ".no-mistakes.yml",
        "--environment",
        "local-no-global",
        "--json",
    ]);

    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], false);
    assert!(plan["fallback_reason"].is_null());
    assert_eq!(plan["selected_tests"].as_array().unwrap().len(), 1);
    assert_eq!(plan["selected_tests"][0]["reasons"][0]["via"][0], "sample");
}

#[test]
fn test_plan_vitest_global_config_fallback_cli_override_wins() {
    let root = fixture("test-plan-config");
    let disabled_output = run(&[
        "test",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        ".no-mistakes.yml",
        "--global-config-fallback",
        "false",
        "--json",
    ]);

    assert!(disabled_output.status.success());
    let disabled: serde_json::Value = serde_json::from_str(&stdout(&disabled_output)).unwrap();
    assert_eq!(disabled["fallback_triggered"], false);
    assert_eq!(disabled["selected_tests"].as_array().unwrap().len(), 1);

    let enabled_output = run(&[
        "test",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        ".no-mistakes.yml",
        "--environment",
        "local-no-global",
        "--global-config-fallback",
        "true",
        "--json",
    ]);

    assert!(enabled_output.status.success());
    let enabled: serde_json::Value = serde_json::from_str(&stdout(&enabled_output)).unwrap();
    assert_eq!(enabled["fallback_triggered"], true);
    assert_eq!(enabled["selected_tests"].as_array().unwrap().len(), 2);
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
fn test_plan_vitest_project_dependency_include_keeps_root_wide_match() {
    let root = fixture("test-plan-config");
    let output = run(&[
        "test",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "config/other.ts",
        "--json",
    ]);

    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], true);
    assert!(plan["fallback_reason"]
        .as_str()
        .unwrap()
        .contains("config-include project dependency changed"));
}

#[test]
fn test_plan_vitest_dependency_all_matches_dot_project_root() {
    let root = fixture("test-plan-root-project");
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
    assert_eq!(plan["fallback_triggered"], true);
    assert!(plan["fallback_reason"]
        .as_str()
        .unwrap()
        .contains("root project dependency changed"));
    assert_eq!(
        plan["selected_tests"][0]["reasons"][0]["changed_file"],
        "source.ts"
    );
}

#[test]
fn test_plan_vitest_excludes_playwright_specs_by_default() {
    let root = fixture("playwright-coverage-alt");
    let output = run(&[
        "test",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "package.json",
        "--json",
    ]);

    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(plan["selected_tests"].as_array().unwrap().is_empty());
}

#[test]
fn test_plan_playwright_global_fallback_discovers_specs_outside_e2e_dirs() {
    let root = fixture("playwright-coverage-alt");
    let output = run(&[
        "test",
        "plan",
        "playwright",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "package.json",
        "--json",
    ]);

    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], true);
    assert_eq!(
        plan["selected_tests"][0]["test_file"],
        "specs/dashboard.spec.ts"
    );
    assert_eq!(
        plan["selected_tests"][0]["reasons"][0]["changed_file"],
        "package.json"
    );
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
    assert!(plan["groups"][0]["limit"].is_null());
    assert_eq!(plan["groups"][1]["type"], "coverage");
    assert!(plan["groups"][1]["limit"].is_null());
    assert_eq!(plan["groups"][1]["selected"][0], "tests/e2e/routes.spec.ts");
    assert_eq!(
        only_reason_via(&plan, "tests/e2e/routes.spec.ts"),
        vec!["dependency", "route"]
    );
}
