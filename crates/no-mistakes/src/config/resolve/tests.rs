use super::resolve_config;
use super::triggers::{resolved_framework_triggers, resolved_vitest_triggers};
use crate::config::v2::schema::{
    NamedFullSuiteTrigger, NoMistakesConfig, PlaywrightAppBinding, Project, RewriteRule,
    TestPlanProjectDependency, TestPlanTargetedProjectDependency,
};
use crate::config::v2::FrontendApp;
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
    assert!(report.full_suite_triggers.iter().any(|entry| {
        entry.framework == "vitest"
            && entry
                .triggers
                .iter()
                .any(|trigger| trigger.name == "postgres-resources")
    }));
    assert!(report.playwright.coverage_routes);
    assert!(report.playwright.coverage_selectors);
}

#[test]
fn resolve_config_json_impl_returns_the_same_named_triggers() {
    let root = named_triggers_fixture();
    let output = crate::napi_api::resolve_config_json_impl(
        crate::napi_api::options::test_json_arg(json!({ "root": root }).to_string()),
    )
    .unwrap();
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    let names: Vec<&str> = report["vitestFullSuiteTriggers"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|trigger| trigger["name"].as_str())
        .collect();
    assert!(names.contains(&"postgres-resources"));
    assert_eq!(
        report["fullSuiteTriggers"][0]["framework"].as_str(),
        Some("vitest")
    );
    assert_eq!(
        report["fullSuiteTriggers"][0]["triggers"][0]["name"].as_str(),
        Some("postgres-resources")
    );
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
    let triggers = resolved_vitest_triggers(&config);
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
            include_changed_tests: None,
        }),
    );
    config.test_plan.vitest.full_suite_triggers.projects.insert(
        "orphan".to_string(),
        TestPlanProjectDependency::Patterns(vec!["!./dist/**".to_string()]),
    );
    config.projects.insert(
        "included".to_string(),
        Project {
            root: Some("packages/app".to_string()),
            include: vec!["lib/**".to_string()],
            ..Project::default()
        },
    );
    config
        .test_plan
        .vitest
        .full_suite_triggers
        .projects
        .insert("included".to_string(), TestPlanProjectDependency::All(true));
    config.projects.insert(
        "workspace".to_string(),
        Project {
            root: Some(".".to_string()),
            ..Project::default()
        },
    );
    config.test_plan.vitest.full_suite_triggers.projects.insert(
        "workspace".to_string(),
        TestPlanProjectDependency::All(true),
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
            include_changed_tests: None,
        });

    let triggers = resolved_vitest_triggers(&config);
    let by_name = triggers
        .iter()
        .map(|trigger| (trigger.name.as_str(), trigger))
        .collect::<std::collections::BTreeMap<_, _>>();
    assert!(!by_name.contains_key("skip"));
    assert!(!by_name.contains_key("orphan"));
    assert_eq!(by_name["generated"].paths, vec!["packages/generated/**"]);
    assert_eq!(by_name["included"].paths, vec!["packages/app/lib/**"]);
    assert_eq!(by_name["workspace"].paths, vec!["**"]);
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
    assert_eq!(by_name["root-app"].include_changed_tests, Some(false));
    assert_eq!(
        by_name["prefixed"].paths,
        vec!["packages/generated/src/**".to_string()]
    );
    assert_eq!(
        by_name["resources"].paths,
        vec!["db/**".to_string(), "!db/tmp/**".to_string()]
    );
    assert_eq!(by_name["resources"].source, "triggers");
    assert_eq!(by_name["resources"].include_changed_tests, Some(false));
    assert_eq!(by_name["generated"].include_changed_tests, None);
}

#[test]
fn resolve_config_reports_non_vitest_named_triggers() {
    let mut config = NoMistakesConfig::default();
    config
        .test_plan
        .python
        .full_suite_triggers
        .triggers
        .push(NamedFullSuiteTrigger {
            name: "schema".to_string(),
            paths: vec!["./db/**".to_string()],
            targets: Vec::new(),
            include_changed_tests: None,
        });
    config
        .projects
        .insert("bare".to_string(), Project::default());
    config.test_plan.jest.full_suite_triggers.projects.insert(
        "bare".to_string(),
        TestPlanProjectDependency::Patterns(vec!["src/**".to_string()]),
    );
    let vitest = resolved_vitest_triggers(&config);
    let frameworks = resolved_framework_triggers(&config);
    assert!(vitest.is_empty());
    assert_eq!(
        frameworks
            .iter()
            .map(|entry| entry.framework)
            .collect::<Vec<_>>(),
        vec!["python", "jest"]
    );
    assert_eq!(frameworks[0].triggers[0].paths, vec!["db/**"]);
    assert_eq!(frameworks[1].triggers[0].paths, vec!["bare/src/**"]);
}

#[test]
fn resolve_config_reports_kotlin_named_triggers() {
    let mut config = NoMistakesConfig::default();
    config
        .test_plan
        .kotlin
        .full_suite_triggers
        .triggers
        .push(NamedFullSuiteTrigger {
            name: "schema".to_string(),
            paths: vec!["./db/**".to_string()],
            targets: Vec::new(),
            include_changed_tests: None,
        });
    let frameworks = resolved_framework_triggers(&config);
    assert_eq!(
        frameworks
            .iter()
            .map(|entry| entry.framework)
            .collect::<Vec<_>>(),
        vec!["kotlin"]
    );
    assert_eq!(frameworks[0].triggers[0].name, "schema");
    assert_eq!(frameworks[0].triggers[0].paths, vec!["db/**"]);
}

#[test]
fn resolve_config_reports_elixir_named_triggers() {
    let mut config = NoMistakesConfig::default();
    config
        .test_plan
        .elixir
        .full_suite_triggers
        .triggers
        .push(NamedFullSuiteTrigger {
            name: "schema".to_string(),
            paths: vec!["./db/**".to_string()],
            targets: Vec::new(),
            include_changed_tests: None,
        });
    let frameworks = resolved_framework_triggers(&config);
    assert_eq!(
        frameworks
            .iter()
            .map(|entry| entry.framework)
            .collect::<Vec<_>>(),
        vec!["elixir"]
    );
    assert_eq!(frameworks[0].triggers[0].name, "schema");
    assert_eq!(frameworks[0].triggers[0].paths, vec!["db/**"]);
}

#[test]
fn resolve_config_reports_dart_named_triggers() {
    let mut config = NoMistakesConfig::default();
    config
        .test_plan
        .dart
        .full_suite_triggers
        .triggers
        .push(NamedFullSuiteTrigger {
            name: "schema".to_string(),
            paths: vec!["./db/**".to_string()],
            targets: Vec::new(),
            include_changed_tests: None,
        });
    let frameworks = resolved_framework_triggers(&config);
    assert_eq!(
        frameworks
            .iter()
            .map(|entry| entry.framework)
            .collect::<Vec<_>>(),
        vec!["dart"]
    );
    assert_eq!(frameworks[0].triggers[0].name, "schema");
    assert_eq!(frameworks[0].triggers[0].paths, vec!["db/**"]);
}

#[test]
fn resolve_playwright_apps_include_effective_rewrites_and_ignore_routes() {
    let mut config = NoMistakesConfig::default();
    config.tests.playwright.frontend_root = Some("top/app".to_string());
    config.tests.playwright.selector_roots = vec!["top/selectors".to_string()];
    config.tests.playwright.ignore_routes = Some(vec!["/admin/**".to_string()]);
    config.tests.playwright.apps.insert(
        "chromium".to_string(),
        PlaywrightAppBinding {
            project: Some("web".to_string()),
            ..PlaywrightAppBinding::default()
        },
    );
    config.tests.playwright.apps.insert(
        "override".to_string(),
        PlaywrightAppBinding {
            project: Some("web".to_string()),
            rewrites: vec![RewriteRule {
                source: "/old".to_string(),
                destination: "/new".to_string(),
            }],
            ignore_routes: Some(vec!["/override/**".to_string()]),
            ..PlaywrightAppBinding::default()
        },
    );
    let apps = [FrontendApp {
        project: Some("web".to_string()),
        root: "web".to_string(),
        route_root: "web/app".to_string(),
        selector_roots: vec!["web".to_string()],
        rewrites: vec![RewriteRule {
            source: "/from-app".to_string(),
            destination: "/to-app".to_string(),
        }],
    }];
    let report = super::resolved_playwright(&config, &apps);
    assert_eq!(report.apps[0].playwright_project, "chromium");
    assert_eq!(report.apps[0].frontend_root.as_deref(), Some("top/app"));
    assert_eq!(report.apps[0].selector_roots, vec!["top/selectors"]);
    assert_eq!(report.apps[0].rewrites[0].source, "/from-app");
    assert_eq!(report.apps[0].ignore_routes, vec!["/admin/**"]);
    assert_eq!(report.apps[1].playwright_project, "override");
    assert_eq!(report.apps[1].rewrites[0].source, "/old");
    assert_eq!(report.apps[1].ignore_routes, vec!["/override/**"]);
}
