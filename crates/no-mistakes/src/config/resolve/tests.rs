use super::resolve_config;
use super::triggers::resolved_triggers;
use crate::config::v2::schema::{
    NamedFullSuiteTrigger, NoMistakesConfig, Project, TestPlanProjectDependency,
    TestPlanTargetedProjectDependency,
};
use serde_json::json;
use std::path::PathBuf;

fn named_triggers_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/config/named-full-suite-triggers")
}

#[test]
fn resolve_config_reports_named_triggers_and_coverage_gates() {
    let report = resolve_config(&named_triggers_fixture(), None).unwrap();
    assert!(report
        .vitest_full_suite_triggers
        .iter()
        .any(|trigger| trigger.name == "postgres-resources" && trigger.source == "triggers"));
    assert!(report.playwright.coverage_routes);
    assert!(report.playwright.coverage_selectors);
}

#[test]
fn resolve_config_json_impl_returns_the_same_named_triggers() {
    let root = named_triggers_fixture();
    let output =
        crate::napi_api::resolve_config_json_impl(json!({ "root": root }).to_string()).unwrap();
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    let names: Vec<&str> = report["vitestFullSuiteTriggers"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|trigger| trigger["name"].as_str())
        .collect();
    assert!(names.contains(&"postgres-resources"));
}

#[test]
fn resolve_config_expands_project_keyed_trigger_paths() {
    let mut config = NoMistakesConfig::default();
    config.projects.insert(
        "generated".to_string(),
        Project {
            root: Some("packages/generated".to_string()),
            ..Project::default()
        },
    );
    config.test_plan.vitest.full_suite_triggers.projects.insert(
        "generated".to_string(),
        TestPlanProjectDependency::Patterns(vec!["src/**".to_string()]),
    );
    let triggers = resolved_triggers(&config);
    assert_eq!(triggers.len(), 1);
    assert_eq!(triggers[0].name, "generated");
    assert_eq!(triggers[0].source, "projects");
    assert_eq!(triggers[0].paths, vec!["packages/generated/src/**"]);
}

#[test]
fn resolve_config_covers_project_trigger_shapes_and_glob_normalization() {
    let mut config = NoMistakesConfig::default();
    config.projects.insert(
        "generated".to_string(),
        Project {
            root: Some("packages/generated".to_string()),
            ..Project::default()
        },
    );
    config.projects.insert(
        "root-app".to_string(),
        Project {
            root: Some(".".to_string()),
            ..Project::default()
        },
    );
    config.projects.insert(
        "skip".to_string(),
        Project {
            root: Some("unused".to_string()),
            ..Project::default()
        },
    );
    config
        .test_plan
        .vitest
        .full_suite_triggers
        .projects
        .insert("skip".to_string(), TestPlanProjectDependency::All(false));
    config.test_plan.vitest.full_suite_triggers.projects.insert(
        "generated".to_string(),
        TestPlanProjectDependency::All(true),
    );
    config.test_plan.vitest.full_suite_triggers.projects.insert(
        "root-app".to_string(),
        TestPlanProjectDependency::Targeted(TestPlanTargetedProjectDependency {
            paths: vec![
                "./src/**".to_string(),
                " !./src/generated/**".to_string(),
                "packages/generated/src/**".to_string(),
            ],
            targets: vec!["unit".to_string()],
        }),
    );
    config.test_plan.vitest.full_suite_triggers.projects.insert(
        "orphan".to_string(),
        TestPlanProjectDependency::Patterns(vec!["!./dist/**".to_string()]),
    );
    config.projects.insert(
        "prefixed".to_string(),
        Project {
            root: Some("packages/generated".to_string()),
            ..Project::default()
        },
    );
    config.test_plan.vitest.full_suite_triggers.projects.insert(
        "prefixed".to_string(),
        TestPlanProjectDependency::Patterns(vec!["packages/generated/src/**".to_string()]),
    );
    config
        .test_plan
        .vitest
        .full_suite_triggers
        .triggers
        .push(NamedFullSuiteTrigger {
            name: "resources".to_string(),
            paths: vec!["./db/**".to_string(), " !./db/tmp/**".to_string()],
            targets: vec!["backend".to_string()],
        });

    let triggers = resolved_triggers(&config);
    let by_name = triggers
        .iter()
        .map(|trigger| (trigger.name.as_str(), trigger))
        .collect::<std::collections::BTreeMap<_, _>>();
    assert!(!by_name.contains_key("skip"));
    assert!(!by_name.contains_key("orphan"));
    assert_eq!(by_name["generated"].paths.len(), 0);
    assert_eq!(by_name["generated"].targets.len(), 0);
    assert_eq!(
        by_name["root-app"].paths,
        vec![
            "src/**".to_string(),
            "!src/generated/**".to_string(),
            "packages/generated/src/**".to_string()
        ]
    );
    assert_eq!(by_name["root-app"].targets, vec!["unit".to_string()]);
    assert_eq!(
        by_name["prefixed"].paths,
        vec!["packages/generated/src/**".to_string()]
    );
    assert_eq!(
        by_name["resources"].paths,
        vec!["db/**".to_string(), "!db/tmp/**".to_string()]
    );
    assert_eq!(by_name["resources"].source, "triggers");
}
