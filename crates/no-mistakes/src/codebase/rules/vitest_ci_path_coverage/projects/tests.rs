use super::*;
use crate::config::v2::schema::{StringOrList, TestProjectPolicy};
use std::collections::BTreeMap;

fn fixture_root(name: &str) -> std::path::PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/vitest-ci-path-coverage")
            .join(name),
    )
}

#[test]
fn project_pattern_helpers_cover_roots_relative_patterns_and_excludes() {
    assert_eq!(project_root_patterns("."), vec!["**"]);
    assert_eq!(
        project_root_patterns(" packages/api/ "),
        vec!["packages/api/**"]
    );
    assert_eq!(
        project_relative_pattern("packages/api", "!./src/**/*.ts"),
        "packages/api/!./src/**/*.ts"
    );
    assert_eq!(
        project_relative_pattern("packages/api", "packages/api/src/**/*.ts"),
        "packages/api/src/**/*.ts"
    );

    let project = ConfigProject {
        config: None,
        policy_name: None,
        runner_project_arg: None,
        scope: None,
        include: vec!["./src/**/*.test.ts".to_string()],
        exclude: vec!["./src/generated/**".to_string()],
    };
    assert_eq!(project_name(&project), "default");
    assert_eq!(
        include_without_excludes(&project),
        vec!["src/**/*.test.ts", "!src/generated/**"]
    );
}

#[test]
fn project_dependency_patterns_cover_all_trigger_shapes() {
    let project = Project {
        root: Some("pkg".to_string()),
        include: vec!["src/**/*.ts".to_string()],
        ..Project::default()
    };
    assert!(
        project_dependency_patterns("pkg", &project, &TestPlanProjectDependency::All(false))
            .is_empty()
    );
    assert_eq!(
        project_dependency_patterns("pkg", &project, &TestPlanProjectDependency::All(true)),
        vec!["pkg/src/**/*.ts"]
    );
    assert_eq!(
        project_dependency_patterns(
            "pkg",
            &Project {
                root: Some("pkg".to_string()),
                ..Project::default()
            },
            &TestPlanProjectDependency::All(true)
        ),
        vec!["pkg/**"]
    );
    assert_eq!(
        project_dependency_patterns(
            "pkg",
            &Project {
                root: Some("pkg".to_string()),
                ..Project::default()
            },
            &TestPlanProjectDependency::Patterns(vec!["!dist/**".to_string()])
        ),
        vec!["pkg/!dist/**"]
    );
}

#[test]
fn needs_config_projects_covers_config_and_policy_branches() {
    let root = fixture_root("fixture");
    let mut config = NoMistakesConfig::default();
    assert!(needs_config_projects(&root, &config));

    config.tests.vitest.configs = Some(StringOrList::One("missing.config.mts".to_string()));
    config.tests.vitest.projects.insert(
        "unit".to_string(),
        TestProjectPolicy {
            include: vec!["src/**/*.test.ts".to_string()],
            exclude: Vec::new(),
            integration_suites: BTreeMap::new(),
        },
    );
    assert!(!needs_config_projects(&root, &config));

    config.tests.vitest.configs = Some(StringOrList::One("vitest.config.mts".to_string()));
    assert!(needs_config_projects(&root, &config));

    config.tests.vitest.configs = Some(StringOrList::One("missing.config.mts".to_string()));
    config
        .tests
        .vitest
        .projects
        .get_mut("unit")
        .unwrap()
        .include
        .clear();
    assert!(needs_config_projects(&root, &config));
}

#[test]
fn coverage_units_cover_missing_project_and_explicit_project_branches() {
    let root = fixture_root("fixture");
    let mut config = NoMistakesConfig::default();
    config
        .test_plan
        .vitest
        .full_suite_triggers
        .projects
        .insert("missing".to_string(), TestPlanProjectDependency::All(true));
    let opts = Options {
        include_vitest_project_globs: Some(false),
        ..Options::default()
    };
    assert!(coverage_units(&root, &config, &opts).unwrap().is_empty());

    let err = coverage_units(
        &root,
        &config,
        &Options {
            explicit_projects_only: true,
            include_full_suite_triggers: Some(false),
            ..Options::default()
        },
    )
    .unwrap_err();
    assert!(err.to_string().contains("explicitProjectsOnly"));

    config.tests.vitest.projects.insert(
        "unit".to_string(),
        TestProjectPolicy {
            include: vec!["src/**/*.test.ts".to_string()],
            exclude: Vec::new(),
            integration_suites: BTreeMap::new(),
        },
    );
    let units = coverage_units(
        &root,
        &config,
        &Options {
            explicit_projects_only: true,
            include_full_suite_triggers: Some(false),
            ..Options::default()
        },
    )
    .unwrap();
    assert_eq!(units[0].source, "test include");
    assert_eq!(units[0].patterns, vec!["src/**/*.test.ts"]);
}

#[test]
fn coverage_units_merges_explicit_projects_without_loading_config_when_not_needed() {
    let root = fixture_root("fixture");
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(StringOrList::One("missing.config.mts".to_string()));
    config.tests.vitest.projects.insert(
        "unit".to_string(),
        TestProjectPolicy {
            include: vec!["src/**/*.test.ts".to_string()],
            exclude: vec!["src/generated/**".to_string()],
            integration_suites: BTreeMap::new(),
        },
    );

    let units = coverage_units(
        &root,
        &config,
        &Options {
            include_full_suite_triggers: Some(false),
            ..Options::default()
        },
    )
    .unwrap();

    assert_eq!(units.len(), 1);
    assert_eq!(
        units[0].patterns,
        vec!["src/**/*.test.ts", "!src/generated/**"]
    );
}

#[test]
fn coverage_units_loads_vitest_config_when_needed() {
    let root = fixture_root("fixture");
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(StringOrList::One("vitest.config.mts".to_string()));

    let units = coverage_units(
        &root,
        &config,
        &Options {
            include_full_suite_triggers: Some(false),
            ..Options::default()
        },
    )
    .unwrap();

    assert_eq!(units[0].project, "backend");
    assert_eq!(units[0].patterns, vec!["src/**/*.test.ts"]);
}
