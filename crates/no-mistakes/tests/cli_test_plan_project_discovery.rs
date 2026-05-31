mod common;

use common::{fixture, run, stdout};

#[test]
fn test_plan_vitest_uses_project_includes_and_targets() {
    let root = fixture("test-plan-project-discovery");
    let output = run(&[
        "test",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "web/storybook/button.stories.tsx",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0]["test_file"], "web/storybook/button.stories.tsx");
    let targets = selected[0]["targets"].as_array().unwrap();
    let mut projects: Vec<&str> = targets
        .iter()
        .map(|target| target["project"].as_str().unwrap())
        .collect();
    projects.sort_unstable();
    assert_eq!(projects, vec!["browser", "stories"]);
    let browser_target = targets
        .iter()
        .find(|target| target["project"] == "browser")
        .unwrap();
    let stories_target = targets
        .iter()
        .find(|target| target["project"] == "stories")
        .unwrap();
    assert_eq!(browser_target["runner"], "vitest");
    assert_eq!(browser_target["config"], "vitest.config.mts");
    assert_eq!(stories_target["config"], "vitest.config.mts");
    assert_eq!(
        browser_target["runner_args"]
            .as_array()
            .unwrap()
            .last()
            .unwrap(),
        "web/storybook/button.stories.tsx"
    );
}

#[test]
fn test_plan_vitest_project_excludes_are_applied() {
    let root = fixture("test-plan-project-discovery");
    let output = run(&[
        "test",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        ".no-mistakes.yml",
        "--environment",
        "all",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let selected: Vec<&str> = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|test| test["test_file"].as_str().unwrap())
        .collect();
    assert_eq!(
        selected,
        vec!["e2e/home.pw.ts", "web/storybook/button.stories.tsx"]
    );
}

#[test]
fn test_plan_playwright_uses_project_match_and_targets() {
    let root = fixture("test-plan-project-discovery");
    let output = run(&[
        "test",
        "plan",
        "playwright",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        ".no-mistakes.yml",
        "--environment",
        "all",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();
    let test_files: Vec<&str> = selected
        .iter()
        .map(|test| test["test_file"].as_str().unwrap())
        .collect();
    assert_eq!(test_files, vec!["e2e/[locale].pw.ts", "e2e/home.pw.ts"]);
    let locale_test = selected
        .iter()
        .find(|test| test["test_file"] == "e2e/[locale].pw.ts")
        .unwrap();
    let locale_target = locale_test["targets"].as_array().unwrap().first().unwrap();
    assert_eq!(
        locale_target["runner_args"]
            .as_array()
            .unwrap()
            .last()
            .unwrap(),
        "e2e/\\[locale\\]\\.pw\\.ts"
    );
    let home_test = selected
        .iter()
        .find(|test| test["test_file"] == "e2e/home.pw.ts")
        .unwrap();
    let targets = home_test["targets"].as_array().unwrap();
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0]["runner"], "playwright");
    assert_eq!(targets[0]["config"], "playwright.config.ts");
    assert_eq!(targets[0]["project"], "chromium");
    assert_eq!(
        targets[0]["base_command"],
        serde_json::json!(["playwright", "test"])
    );
}

#[test]
fn test_plan_vitest_keeps_duplicate_project_targets_from_distinct_configs() {
    let root = fixture("test-plan-multi-vitest-configs");
    // The shared test is selected by explicit policy replacement, not by the
    // original per-config include globs. Keep those includes intentionally
    // different so both matching configs must survive replacement.
    let output = run(&[
        "test",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "src/shared.test.ts",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0]["test_file"], "src/shared.test.ts");
    let mut configs: Vec<&str> = selected[0]["targets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|target| target["config"].as_str().unwrap())
        .collect();
    configs.sort_unstable();
    assert_eq!(configs, vec!["vitest.browser.mts", "vitest.node.mts"]);
    assert!(selected[0]["targets"]
        .as_array()
        .unwrap()
        .iter()
        .all(|target| target["project"] == "shared"));
}

#[test]
fn test_plan_playwright_top_level_config_name_does_not_emit_project_arg() {
    let root = fixture("test-plan-playwright-root-name");
    let output = run(&[
        "test",
        "plan",
        "playwright",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        ".no-mistakes.yml",
        "--environment",
        "all",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0]["test_file"], "e2e/root-name.spec.ts");
    let target = selected[0]["targets"].as_array().unwrap().first().unwrap();
    assert!(target["project"].is_null());
    assert!(!target["runner_args"]
        .as_array()
        .unwrap()
        .iter()
        .any(|arg| arg == "--project"));
}
