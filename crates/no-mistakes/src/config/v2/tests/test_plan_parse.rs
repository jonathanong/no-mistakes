use crate::config::v2::schema::{NoMistakesConfig, TestPlanIgnoredChangedTestsFramework};

#[test]
fn test_plan_deprecated_dependencies_key_still_parsed() {
    let cfg: NoMistakesConfig = serde_yaml::from_str(
        r#"
test_plan:
  playwright:
    dependencies:
      ignore_changed_tests:
        - vitest
      projects:
        web: true
  vitest:
    dependencies:
      ignoreChangedTests:
        - playwright
"#,
    )
    .unwrap();

    // Old `dependencies` key is still deserialized correctly.
    assert_eq!(
        cfg.test_plan
            .playwright
            .full_suite_triggers
            .ignore_changed_tests,
        vec![TestPlanIgnoredChangedTestsFramework::Vitest]
    );
    assert_eq!(
        cfg.test_plan
            .vitest
            .full_suite_triggers
            .ignore_changed_tests,
        vec![TestPlanIgnoredChangedTestsFramework::Playwright]
    );
    // Deprecated flag is set so the load path can emit a warning.
    assert!(cfg.test_plan.playwright.deprecated_dependencies_key);
    assert!(cfg.test_plan.vitest.deprecated_dependencies_key);
}

#[test]
fn test_plan_full_suite_triggers_key_parsed() {
    let cfg: NoMistakesConfig = serde_yaml::from_str(
        r#"
test_plan:
  playwright:
    fullSuiteTriggers:
      projects:
        web: true
  vitest:
    fullSuiteTriggers:
      ignoreChangedTests:
        - playwright
"#,
    )
    .unwrap();

    assert!(cfg
        .test_plan
        .playwright
        .full_suite_triggers
        .projects
        .contains_key("web"));
    assert_eq!(
        cfg.test_plan
            .vitest
            .full_suite_triggers
            .ignore_changed_tests,
        vec![TestPlanIgnoredChangedTestsFramework::Playwright]
    );
    // New key does not set the deprecated flag.
    assert!(!cfg.test_plan.playwright.deprecated_dependencies_key);
    assert!(!cfg.test_plan.vitest.deprecated_dependencies_key);
}
