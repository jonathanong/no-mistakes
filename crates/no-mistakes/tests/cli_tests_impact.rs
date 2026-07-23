use std::path::PathBuf;
use std::process::{Command, Output};

#[path = "cli_tests_impact/vitest_setup.rs"]
mod vitest_setup;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis")
            .join(name)
            .join("fixture"),
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

fn plan_for(root: &std::path::Path, changed_file: &str) -> serde_json::Value {
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        changed_file,
        "--json",
    ]);

    assert!(output.status.success());
    serde_json::from_str(&stdout(&output)).unwrap()
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
fn tests_plan_json_outputs_impacted_tests() {
    let root = fixture("tests-impact");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "c.mts",
        "--json",
    ]);

    assert!(output.status.success());
    let json_str = stdout(&output);
    let plan: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(plan["fallback_triggered"], false);

    let selected = plan["selected_tests"].as_array().unwrap();
    assert_eq!(selected.len(), 2);
    let mut names: Vec<&str> = selected
        .iter()
        .map(|t| t["test_file"].as_str().unwrap())
        .collect();
    names.sort_unstable();
    assert_eq!(names, vec!["a.test.mts", "dynamic.test.mts"]);

    let a_test = selected
        .iter()
        .find(|t| t["test_file"] == "a.test.mts")
        .unwrap();
    assert_eq!(a_test["confidence"], "high");

    let dynamic_test = selected
        .iter()
        .find(|t| t["test_file"] == "dynamic.test.mts")
        .unwrap();
    assert_eq!(dynamic_test["confidence"], "medium");
}

#[test]
fn tests_plan_commands_format_requires_execution_targets() {
    let root = fixture("tests-impact");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "c.mts",
        "--format",
        "commands",
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(
        "`tests plan --format commands` requires selected tests to include framework execution targets"
    ));
    assert!(stdout(&output).is_empty());
}

#[test]
fn tests_impact_commands_format_requires_execution_targets() {
    let root = fixture("tests-impact");
    let output = run(&[
        "tests",
        "impact",
        "--root",
        root.to_str().unwrap(),
        "c.mts",
        "--format",
        "commands",
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(
        "`tests impact --format commands` requires selected tests to include framework execution targets"
    ));
    assert!(stdout(&output).is_empty());
}

#[test]
fn tests_plan_ignores_deleted_changed_files() {
    let root = fixture("tests-impact");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "deleted.mts",
        "--changed-file",
        "c.mts",
        "--json",
    ]);

    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();

    assert_eq!(plan["fallback_triggered"], false);
    assert!(plan["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .all(|warning| warning["type"] != "file-not-found"));

    let selected = plan["selected_tests"].as_array().unwrap();
    let mut names: Vec<&str> = selected
        .iter()
        .map(|test| test["test_file"].as_str().unwrap())
        .collect();
    names.sort_unstable();
    assert_eq!(names, vec!["a.test.mts", "dynamic.test.mts"]);
    assert!(selected.iter().all(|test| {
        test["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .all(|reason| reason["changed_file"] != "deleted.mts")
    }));
}

#[test]
fn tests_plan_keeps_changed_broken_symlink_entries() {
    let root = fixture("tests-impact");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "broken.test.mts",
        "--json",
    ]);

    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();

    assert_eq!(plan["fallback_triggered"], false);
    let selected = plan["selected_tests"].as_array().unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0]["test_file"], "broken.test.mts");
    assert_eq!(selected[0]["reasons"][0]["changed_file"], "broken.test.mts");
}

#[test]
fn tests_plan_ignores_changed_paths_below_file_components() {
    let root = fixture("tests-impact");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "package.json/deleted.test.mts",
        "--json",
    ]);

    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();

    assert_eq!(plan["fallback_triggered"], false);
    assert!(plan["selected_tests"].as_array().unwrap().is_empty());
    assert!(plan["warnings"].as_array().unwrap().is_empty());
}

#[test]
fn tests_plan_matches_playwright_route_when_page_dependency_changes() {
    let root = fixture("playwright-impact-routing");
    let plan = plan_for(&root, "web/components/UserCard.tsx");

    assert_eq!(plan["fallback_triggered"], false);
    let selected = plan["selected_tests"].as_array().unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0]["test_file"], "tests/e2e/routes.spec.ts");
    assert_eq!(
        only_reason_via(&plan, "tests/e2e/routes.spec.ts"),
        vec!["dependency", "route"]
    );
}

#[test]
fn tests_plan_matches_playwright_route_when_parent_layout_changes() {
    let root = fixture("playwright-impact-routing");
    let plan = plan_for(&root, "web/app/users/layout.tsx");

    assert_eq!(plan["fallback_triggered"], false);
    let selected = plan["selected_tests"].as_array().unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0]["test_file"], "tests/e2e/routes.spec.ts");
    assert_eq!(
        only_reason_via(&plan, "tests/e2e/routes.spec.ts"),
        vec!["layout", "route"]
    );
}

#[test]
fn tests_plan_does_not_fallback_when_next_proxy_changes_by_default() {
    let root = fixture("playwright-impact-routing");
    let plan = plan_for(&root, "web/proxy.ts");

    assert_eq!(plan["fallback_triggered"], false);
    assert!(plan["fallback_reason"].is_null());
    assert!(plan["selected_tests"].as_array().unwrap().is_empty());
}

#[test]
fn tests_plan_does_not_fallback_for_src_app_next_project_proxy_by_default() {
    let root = fixture("playwright-impact-routing");
    let plan = plan_for(&root, "web/src-only/proxy.ts");

    assert_eq!(plan["fallback_triggered"], false);
    assert!(plan["fallback_reason"].is_null());
    assert!(plan["selected_tests"].as_array().unwrap().is_empty());
}

#[test]
fn tests_plan_does_not_treat_arbitrary_nested_proxy_as_global_config() {
    let root = fixture("playwright-impact-routing");
    let plan = plan_for(&root, "web/nested/proxy.ts");

    assert_eq!(plan["fallback_triggered"], false);
    assert!(plan["selected_tests"].as_array().unwrap().is_empty());
}

#[test]
fn tests_plan_paths_outputs_newline_separated_paths() {
    let root = fixture("tests-impact");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "c.mts",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    let paths_str = stdout(&output);
    let mut paths: Vec<&str> = paths_str.lines().collect();
    paths.sort();
    assert_eq!(paths, vec!["a.test.mts", "dynamic.test.mts"]);
}

#[test]
fn tests_plan_md_outputs_markdown_table() {
    let root = fixture("tests-impact");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "c.mts",
        "--format",
        "md",
    ]);

    assert!(output.status.success());
    let md = stdout(&output);
    assert!(md.contains("# 🧪 Test Impact Analysis"));
    assert!(md.contains("## Selected Tests (Total: 2)"));
    assert!(md.contains("| Test File | Confidence | Reason / Impact Chain |"));
    assert!(md.contains("a.test.mts"));
    assert!(md.contains("dynamic.test.mts"));
}

#[test]
fn tests_plan_does_not_fallback_on_package_json_by_default() {
    let root = fixture("tests-impact");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "package.json",
        "--json",
    ]);

    assert!(output.status.success());
    let json_str = stdout(&output);
    let plan: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(plan["fallback_triggered"], false);
    assert!(plan["fallback_reason"].is_null());
    assert!(plan["selected_tests"].as_array().unwrap().is_empty());
}

#[test]
fn tests_plan_can_opt_into_global_config_fallback() {
    let root = fixture("tests-impact");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "package.json",
        "--global-config-fallback",
        "true",
        "--json",
    ]);

    assert!(output.status.success());
    let json_str = stdout(&output);
    let plan: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(plan["fallback_triggered"], true);
    assert!(plan["fallback_reason"]
        .as_str()
        .unwrap()
        .contains("Global configuration file changed"));

    let selected = plan["selected_tests"].as_array().unwrap();
    // It should select all tests in this fixture
    assert_eq!(selected.len(), 2);
    let mut names: Vec<&str> = selected
        .iter()
        .map(|t| t["test_file"].as_str().unwrap())
        .collect();
    names.sort_unstable();
    assert_eq!(names, vec!["a.test.mts", "dynamic.test.mts"]);
    for t in selected {
        assert_eq!(t["confidence"], "high");
    }
}

#[test]
fn tests_plan_global_config_fallback_disabled_without_framework() {
    let root = fixture("tests-impact");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        ".no-mistakes.yml",
        "--global-config-fallback",
        "false",
        "--json",
    ]);

    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();

    assert_eq!(plan["fallback_triggered"], false);
    assert!(plan["fallback_reason"].is_null());
    assert!(plan["selected_tests"].as_array().unwrap().is_empty());
}

#[test]
fn tests_why_displays_dependency_path() {
    let root = fixture("tests-impact");
    let output = run(&[
        "tests",
        "why",
        "a.test.mts",
        "--changed",
        "c.mts",
        "--root",
        root.to_str().unwrap(),
    ]);

    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.contains("Path from `c.mts` to `a.test.mts`"));
    assert!(text.contains("c.mts"));
    assert!(text.contains("b.mts"));
    assert!(text.contains("a.mts"));
    assert!(text.contains("a.test.mts"));
}

#[test]
fn tests_plan_nested_package_json_does_not_trigger_fallback() {
    let root = fixture("tests-impact");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "nested/package.json",
        "--json",
    ]);

    assert!(output.status.success());
    let json_str = stdout(&output);
    let plan: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(plan["fallback_triggered"], false);
    assert!(plan["warnings"].as_array().unwrap().is_empty());
}

#[test]
fn tests_plan_head_requires_base() {
    let root = fixture("tests-impact");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--head",
        "main",
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr.clone()).unwrap();
    assert!(stderr.contains("--base"));
}
