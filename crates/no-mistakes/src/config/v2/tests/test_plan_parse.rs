use crate::config::v2::schema::{NoMistakesConfig, TestPlanIgnoredChangedTestsFramework};

#[test]
fn test_plan_dependencies_ignore_changed_tests_parsed() {
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

    assert_eq!(
        cfg.test_plan.playwright.dependencies.ignore_changed_tests,
        vec![TestPlanIgnoredChangedTestsFramework::Vitest]
    );
    assert_eq!(
        cfg.test_plan.vitest.dependencies.ignore_changed_tests,
        vec![TestPlanIgnoredChangedTestsFramework::Playwright]
    );
}
