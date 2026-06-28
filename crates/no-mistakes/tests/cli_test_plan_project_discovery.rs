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
fn test_plan_vitest_commands_format_uses_execution_targets() {
    let root = fixture("test-plan-project-discovery");
    let output = run(&[
        "test",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "web/storybook/button.stories.tsx",
        "--format",
        "commands",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = stdout(&output);
    assert!(stdout.contains(
        "vitest --config vitest.config.mts --project browser web/storybook/button.stories.tsx"
    ));
    assert!(stdout.contains(
        "vitest --config vitest.config.mts --project stories web/storybook/button.stories.tsx"
    ));
}

#[test]
fn test_targets_vitest_reports_project_commands() {
    let root = fixture("test-plan-project-discovery");
    let output = run(&[
        "test",
        "targets",
        "vitest",
        "web/storybook/button.stories.tsx",
        "--root",
        root.to_str().unwrap(),
        "--format",
        "commands",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = stdout(&output);
    assert!(stdout.contains(
        "vitest --config vitest.config.mts --project browser web/storybook/button.stories.tsx"
    ));
    assert!(stdout.contains(
        "vitest --config vitest.config.mts --project stories web/storybook/button.stories.tsx"
    ));
}

#[test]
fn test_targets_vitest_commands_format_requires_unmatched_files() {
    let root = fixture("test-plan-project-discovery");
    let output = run(&[
        "test",
        "targets",
        "vitest",
        "web/storybook/button.stories.tsx",
        "web/app/page.tsx",
        "--root",
        root.to_str().unwrap(),
        "--format",
        "commands",
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("warning: web/app/page.tsx"));
    assert!(stderr
        .contains("`tests targets --format commands` requires all requested files to be owned"));
    assert!(stdout(&output).is_empty());
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

#[test]
fn test_plan_playwright_nested_configs_scope_targets_to_owning_config() {
    // Two configs share the `chromium` project name; the credentialed config's
    // testDir is nested inside the standard config's broad testDir. A spec under
    // the credentialed testDir must produce exactly one target for the
    // credentialed config (not a duplicate for the standard config), and the
    // standard spec must map to the standard config.
    let root = fixture("test-plan-playwright-nested-configs");
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

    let credentialed = selected
        .iter()
        .find(|test| test["test_file"] == "playwright/credentialed/chat.spec.mts")
        .unwrap();
    let credentialed_targets = credentialed["targets"].as_array().unwrap();
    assert_eq!(credentialed_targets.len(), 1);
    assert_eq!(
        credentialed_targets[0]["config"],
        "playwright.credentialed.config.mts"
    );
    assert_eq!(credentialed_targets[0]["project"], "chromium");

    let home = selected
        .iter()
        .find(|test| test["test_file"] == "playwright/home.spec.mts")
        .unwrap();
    let home_targets = home["targets"].as_array().unwrap();
    assert_eq!(home_targets.len(), 1);
    assert_eq!(home_targets[0]["config"], "playwright.config.mts");
    assert_eq!(home_targets[0]["project"], "chromium");
}

#[test]
fn test_plan_swift_uses_packages_and_dependency_graph() {
    let root = fixture("swift-test-plan");
    let output = run(&[
        "test",
        "plan",
        "swift",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "backend/api/feeds.mts",
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
        vec![
            "swift-clients/core/Tests/VouchaCoreTests/APIClientTests.swift",
            "swift-clients/ui/Tests/VouchaUITests/RSSFeedListViewModelTests.swift",
        ]
    );
    let core_targets = plan["selected_tests"][0]["targets"].as_array().unwrap();
    let core = core_targets
        .iter()
        .find(|target| target["project"] == "VouchaCoreTests")
        .expect("SwiftPM test target should be emitted");
    assert_eq!(core["runner"], "swift");
    assert_eq!(core["config"], "swift-clients/core");
    assert_eq!(core["base_command"], serde_json::json!(["swift", "test"]));
    assert_eq!(
        core["runner_args"],
        serde_json::json!([
            "--package-path",
            "swift-clients/core",
            "--filter",
            "VouchaCoreTests"
        ])
    );
    let via: Vec<&str> = plan["selected_tests"][0]["reasons"][0]["via"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    assert_eq!(via, vec!["http", "swift"]);
}

#[test]
fn test_plan_swift_direct_and_coverage_error() {
    let root = fixture("swift-test-plan");
    let direct = run(&[
        "test",
        "plan",
        "swift",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "swift-clients/ui/Tests/VouchaUITests/RSSFeedListViewModelTests.swift",
        "--json",
    ]);
    assert!(
        direct.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&direct.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&direct)).unwrap();
    assert_eq!(
        plan["groups"][0]["selected"],
        serde_json::json!(["swift-clients/ui/Tests/VouchaUITests/RSSFeedListViewModelTests.swift"])
    );

    let coverage = run(&[
        "test",
        "plan",
        "swift",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "backend/api/feeds.mts",
        "--environment",
        "coverage-only",
        "--json",
    ]);
    assert!(!coverage.status.success());
    assert!(String::from_utf8_lossy(&coverage.stderr)
        .contains("swift test plans do not support the coverage group"));
}
