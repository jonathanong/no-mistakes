use super::{validate_playwright_selector_wrappers, validate_v2_config};
use crate::config::v2::schema::{
    NoMistakesConfig, PlaywrightSelectorWrapper, Project, RuleDef, TestPlanProjectDependency,
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
    let cases = [
        (
            Vec::new(),
            vec!["unit".to_string()],
            "paths must not be empty",
        ),
        (
            vec!["src/**".to_string()],
            Vec::new(),
            "targets must not be empty",
        ),
        (
            vec!["src/**".to_string()],
            vec![" ".to_string()],
            "targets[0] must not be blank",
        ),
        (
            vec!["[".to_string()],
            vec!["unit".to_string()],
            "contains invalid glob",
        ),
        (
            vec!["!".to_string()],
            vec!["unit".to_string()],
            "paths[0] must not be blank",
        ),
    ];
    for (paths, targets, expected) in cases {
        let mut config = NoMistakesConfig::default();
        config
            .projects
            .insert("app".to_string(), Project::default());
        config.test_plan.vitest.full_suite_triggers.projects.insert(
            "app".to_string(),
            TestPlanProjectDependency::Targeted(TestPlanTargetedProjectDependency {
                paths,
                targets,
            }),
        );
        let error = validate_v2_config(&config, Path::new("config.yml"))
            .unwrap_err()
            .to_string();
        assert!(error.contains(expected), "{error}");
        assert!(error.contains("config.yml.testPlan.vitest.fullSuiteTriggers.projects.app"));
    }

    let mut config = NoMistakesConfig::default();
    config.test_plan.vitest.full_suite_triggers.projects.insert(
        "missing".to_string(),
        TestPlanProjectDependency::Targeted(TestPlanTargetedProjectDependency {
            paths: vec!["src/**".to_string()],
            targets: vec!["unit".to_string()],
        }),
    );
    let error = validate_v2_config(&config, Path::new("config.yml"))
        .unwrap_err()
        .to_string();
    assert!(error.contains(
        "config.yml.testPlan.vitest.fullSuiteTriggers.projects.missing references missing top-level projects.missing"
    ));

    let mut legacy = NoMistakesConfig::default();
    legacy
        .test_plan
        .vitest
        .full_suite_triggers
        .projects
        .insert("missing".to_string(), TestPlanProjectDependency::All(true));
    validate_v2_config(&legacy, Path::new("config.yml")).unwrap();
}

#[test]
fn project_and_rule_globs_surface_validation_context() {
    let mut project_config = NoMistakesConfig::default();
    project_config.projects.insert(
        "app".to_string(),
        Project {
            include: vec!["[".to_string()],
            ..Default::default()
        },
    );
    let project_error = validate_v2_config(&project_config, Path::new("config.yml"))
        .unwrap_err()
        .to_string();
    assert!(
        project_error.contains("projects.app.include contains invalid glob `[`"),
        "{project_error}"
    );

    let mut rule_config = NoMistakesConfig::default();
    rule_config.rules.push(RuleDef {
        rule: "unrelated-rule".to_string(),
        exclude: vec!["[".to_string()],
        ..Default::default()
    });
    let rule_error = validate_v2_config(&rule_config, Path::new("config.yml"))
        .unwrap_err()
        .to_string();
    assert!(
        rule_error.contains("rules[0].exclude contains invalid glob `[`"),
        "{rule_error}"
    );
}
