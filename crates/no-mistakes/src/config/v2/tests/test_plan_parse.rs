use crate::config::v2::schema::{
    NoMistakesConfig, TestPlanFrameworkConfig, TestPlanIgnoredChangedTestsFramework,
};

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

#[test]
fn test_plan_full_suite_triggers_wins_when_both_keys_present() {
    // When both fullSuiteTriggers and the deprecated dependencies key are
    // present, fullSuiteTriggers takes precedence and deprecated flag is false.
    let cfg: NoMistakesConfig = serde_yaml::from_str(
        r#"
test_plan:
  playwright:
    fullSuiteTriggers:
      projects:
        web: true
    dependencies:
      projects:
        other: true
"#,
    )
    .unwrap();

    // fullSuiteTriggers wins — "web" project present, "other" is ignored.
    assert!(cfg
        .test_plan
        .playwright
        .full_suite_triggers
        .projects
        .contains_key("web"));
    assert!(!cfg
        .test_plan
        .playwright
        .full_suite_triggers
        .projects
        .contains_key("other"));
    // fullSuiteTriggers was present so deprecated flag must be false.
    assert!(!cfg.test_plan.playwright.deprecated_dependencies_key);
}

#[test]
fn test_plan_framework_config_dependencies_alias_returns_full_suite_triggers() {
    // The `.dependencies()` method is a backward-compat alias for
    // `.full_suite_triggers` used by older call-sites.
    let cfg = TestPlanFrameworkConfig::default();
    // The alias must return the same reference as full_suite_triggers.
    assert_eq!(
        cfg.dependencies().ignore_changed_tests,
        cfg.full_suite_triggers.ignore_changed_tests
    );
    assert_eq!(
        cfg.dependencies().projects.len(),
        cfg.full_suite_triggers.projects.len()
    );
}

#[test]
fn test_plan_framework_config_deserializes_from_json() {
    // Exercise the JSON deserializer monomorphization of TestPlanFrameworkConfig.
    let json = serde_json::json!({
        "fullSuiteTriggers": {
            "projects": { "web": true }
        }
    });
    let cfg: TestPlanFrameworkConfig = serde_json::from_value(json).unwrap();
    assert!(cfg.full_suite_triggers.projects.contains_key("web"));
    assert!(!cfg.deprecated_dependencies_key);
}

#[test]
fn test_plan_group_sample_when_limited_parsed() {
    let cfg: NoMistakesConfig = serde_yaml::from_str(
        r#"
test_plan:
  vitest:
    environments:
      camel:
        groups:
          - type: sample
            sampleWhenLimited: true
      snake:
        groups:
          - type: sample
            sample_when_limited: true
"#,
    )
    .unwrap();

    assert!(cfg.test_plan.vitest.environments["camel"].groups[0].sample_when_limited);
    assert!(cfg.test_plan.vitest.environments["snake"].groups[0].sample_when_limited);
}
