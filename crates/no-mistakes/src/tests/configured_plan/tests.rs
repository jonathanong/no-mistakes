use super::*;
use no_mistakes::config::v2::schema::{
    NoMistakesConfig, Project, StringOrList, TestPlanIgnoredChangedTestsFramework,
    TestPlanProjectDependency,
};

#[test]
fn dependency_trigger_ignores_changed_test_discovery_errors_for_source_changes() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/test-discovery-policy-fallback/fixture");
    let mut config = NoMistakesConfig::default();
    config.projects.insert(
        "src".to_string(),
        Project {
            root: Some("src".to_string()),
            ..Default::default()
        },
    );
    config.tests.vitest.configs = Some(StringOrList::One("missing.vitest.config.mts".to_string()));
    config
        .test_plan
        .vitest
        .full_suite_triggers
        .ignore_changed_tests = vec![TestPlanIgnoredChangedTestsFramework::Vitest];
    config
        .test_plan
        .vitest
        .full_suite_triggers
        .projects
        .insert("src".to_string(), TestPlanProjectDependency::All(true));

    let trigger = dependency_trigger(
        &root,
        &config,
        TestFramework::Vitest,
        &[root.join("src/component.mts")],
    )
    .unwrap();

    assert!(trigger.is_some());
}
