use super::{validate_playwright_selector_wrappers, validate_v2_config};
use crate::config::v2::schema::{
    NoMistakesConfig, PlaywrightSelectorWrapper, TestPlanProjectDependency,
    TestPlanTargetedProjectDependency,
};
use std::path::Path;

fn wrapper(module: &str, export: &str, test_id_argument: usize) -> PlaywrightSelectorWrapper {
    PlaywrightSelectorWrapper {
        module: module.to_string(),
        export: export.to_string(),
        test_id_argument,
    }
}

#[test]
fn selector_wrapper_identity_fields_must_not_be_blank() {
    let module_error = validate_playwright_selector_wrappers(&[wrapper(" ", "find", 0)])
        .unwrap_err()
        .to_string();
    assert!(module_error.contains(".module must not be blank"));

    let export_error = validate_playwright_selector_wrappers(&[wrapper("./helpers", " ", 0)])
        .unwrap_err()
        .to_string();
    assert!(export_error.contains(".export must not be blank"));
}

#[test]
fn selector_wrapper_duplicate_arguments_must_not_conflict() {
    validate_playwright_selector_wrappers(&[
        wrapper("./helpers", "find", 1),
        wrapper("./helpers", "find", 1),
    ])
    .unwrap();

    let error = validate_playwright_selector_wrappers(&[
        wrapper("./helpers", "find", 0),
        wrapper("./helpers", "find", 1),
    ])
    .unwrap_err()
    .to_string();
    assert!(error.contains("conflicting testIdArgument values 0 and 1"));
}

#[test]
fn targeted_full_suite_trigger_validates_paths_and_targets() {
    let mut config = NoMistakesConfig::default();
    config.test_plan.vitest.full_suite_triggers.projects.insert(
        "app".to_string(),
        TestPlanProjectDependency::Targeted(TestPlanTargetedProjectDependency {
            paths: vec!["!".to_string()],
            targets: vec![" ".to_string()],
        }),
    );
    let error = validate_v2_config(&config, Path::new("config.yml"))
        .unwrap_err()
        .to_string();
    assert!(error.contains("config.yml.testPlan.vitest.fullSuiteTriggers.projects.app.paths[0]"));
}
