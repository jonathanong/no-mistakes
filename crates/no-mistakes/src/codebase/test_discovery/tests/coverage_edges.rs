use std::path::{Path, PathBuf};

use crate::codebase::test_discovery::{
    discovered_test_globs, discovered_test_globs_from_visible, TestRunner,
};
use crate::codebase::ts_resolver::TsConfig;
use crate::config::v2::schema::{
    DotnetProjectConfig, NoMistakesConfig, StringOrList, TestProjectPolicy,
};
use crate::integration_tests::types::ConfigProject;

fn edges_root(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/coverage-region-edges")
        .join(name)
}

fn project(include: &str, exclude: &str) -> ConfigProject {
    ConfigProject {
        config: None,
        workspace: false,
        policy_name: None,
        runner_project_arg: None,
        scope: None,
        include: vec![include.to_string()],
        exclude: if exclude.is_empty() {
            Vec::new()
        } else {
            vec![exclude.to_string()]
        },
        matcher_warnings: Vec::new(),
        vitest_setup: Vec::new(),
        vitest_concurrent: false,
        declared_deadlines: Default::default(),
    }
}

#[test]
fn discovered_globs_report_broken_tsconfig_and_missing_dotnet_projects() {
    let root = super::fixture_root("test-discovery-vitest-defaults");
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(StringOrList::One("vitest.config.mts".to_string()));
    assert_eq!(
        discovered_test_globs(&root, &config, TestRunner::Vitest).unwrap(),
        Some(vec!["src/default.test.ts".to_string()])
    );

    let broken_fixture =
        crate::test_support::materialize_saved_fixture(&edges_root("broken-tsconfig"));
    let error = discovered_test_globs(
        broken_fixture.path(),
        &NoMistakesConfig::default(),
        TestRunner::Vitest,
    )
    .unwrap_err();
    assert!(error.to_string().contains("loading tsconfig"), "{error}");

    let mut dotnet = NoMistakesConfig::default();
    dotnet.tests.dotnet.projects.insert(
        "missing".to_string(),
        DotnetProjectConfig {
            project: "src/Missing.csproj".to_string(),
            include: Vec::new(),
            exclude: Vec::new(),
            test: true,
        },
    );
    let error = discovered_test_globs_from_visible(
        &root,
        &dotnet,
        TestRunner::Dotnet,
        &[],
        &TsConfig::default(),
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("could not be resolved"),
        "{error}"
    );
    assert_eq!(TestRunner::from_name("dotnet"), Some(TestRunner::Dotnet));
}

#[test]
fn invalid_project_exclude_is_dropped_by_public_filters() {
    let root = super::fixture_root("test-discovery-vitest-defaults");
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.projects.insert(
        "bad".to_string(),
        TestProjectPolicy {
            include: vec!["src/**/*.ts".to_string()],
            exclude: vec!["[".to_string()],
            ..Default::default()
        },
    );
    let projects = super::runner_projects_lossy(&root, &config, TestRunner::Vitest);
    assert!(projects.iter().any(|project| {
        project.policy_name.as_deref() == Some("bad")
            && project.exclude.iter().any(|pattern| pattern == "[")
    }));
    assert_eq!(
        super::super::project_filters(&root, &NoMistakesConfig::default()).len(),
        super::super::project_filters(&root, &config).len()
    );
}

#[test]
fn swift_package_choice_and_unscoped_files_use_declared_boundaries() {
    let mut config = NoMistakesConfig::default();
    config.tests.swift.packages = vec!["clients/core".to_string(), "clients".to_string()];
    config.tests.swift.projects.insert(
        "clients/core".to_string(),
        TestProjectPolicy {
            include: vec!["clients/core/Tests/AppTests/AppTests.swift".to_string()],
            ..Default::default()
        },
    );
    let projects = super::super::swift_projects::swift_projects_from_facts(
        Path::new("/repo"),
        &config,
        &crate::codebase::swift::SwiftFactMap::default(),
    );
    let chosen = projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("clients/core"))
        .unwrap();
    assert_eq!(chosen.config.as_deref(), Some("clients/core"));

    let swift_fixture =
        crate::test_support::materialize_saved_fixture(&edges_root("swift-unscoped"));
    let root = swift_fixture.path();
    let discovered = super::discover_from_projects(
        root,
        &NoMistakesConfig::default(),
        TestRunner::Swift,
        vec![project("src/App.swift", "")],
    )
    .unwrap();
    assert!(!discovered.tests.is_empty());
    assert!(!discovered.used_fallback);
    let target = discovered
        .targets_by_path
        .values()
        .next()
        .unwrap()
        .iter()
        .next()
        .unwrap();
    assert_eq!(
        target.base_command,
        ["swift".to_string(), "test".to_string()]
    );
    assert!(target.config.is_none());
}

#[test]
fn language_fallback_and_go_package_dir_cover_remaining_shapes() {
    assert!(super::super::fallback_runner_match(
        TestRunner::Python,
        "pkg/tests/helper.py"
    ));
    assert!(!super::super::fallback_runner_match(
        TestRunner::Swift,
        "pkg/Tests/notes.txt"
    ));
    let target =
        super::super::targets::target_for(TestRunner::Go, None, false, None, "widget_test.go");
    assert_eq!(target.runner_args, vec!["./.".to_string()]);
}
