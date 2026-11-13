use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::codebase::test_discovery::{
    discover_tests_from_prepared_projects, discovered_test_globs,
    discovered_test_globs_from_visible, TestRunner,
};
use crate::codebase::ts_resolver::{TsConfig, TsConfigCatalog};
use crate::codebase::ts_source::{FileInventory, SourceStore};
use crate::config::v2::schema::{
    DotnetProjectConfig, NoMistakesConfig, StringOrList, TestProjectPolicy,
};
use crate::integration_tests::types::{ConfigProject, VitestSetupDependency, VitestSetupField};

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

fn setup(reparse: bool, base: PathBuf) -> VitestSetupDependency {
    VitestSetupDependency {
        field: VitestSetupField::SetupFiles,
        specifier: Some("./setup.ts".to_string()),
        needs_final_catalog_reparse: reparse,
        unresolved_config_extends: None,
        config_extends_provenance: false,
        resolved_path: None,
        resolution_base: base,
        declaration_path: PathBuf::new(),
        declaration_line: 1,
        trigger_paths: Default::default(),
        resolver_candidate_paths: Default::default(),
        conservative_specifiers: Default::default(),
        transitive_trigger_paths: Default::default(),
    }
}

fn prepared(root: &Path) -> super::super::PreparedTestProjects {
    super::super::PreparedTestProjects {
        root: root.to_path_buf(),
        projects: BTreeMap::new(),
        discovery_files: Vec::new(),
        graph_facts: crate::codebase::ts_source::facts::TsFactMap::new(),
        graph_route_occurrences: BTreeMap::new(),
        dotnet_facts: None,
        swift_facts: None,
        cargo_targets: None,
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
fn prepared_projects_cover_cargo_vitest_and_filter_failures() {
    let root = edges_root("broken-tsconfig");
    let config = NoMistakesConfig::default();
    let tsconfig = TsConfig::default();
    let mut prepared = prepared(&root);
    let empty = discover_tests_from_prepared_projects(
        &root,
        &config,
        TestRunner::Cargo,
        &prepared,
        &[],
        &tsconfig,
    )
    .unwrap();
    assert!(empty.tests.is_empty());

    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&[])));
    prepared.cargo_targets = Some(super::super::cargo_targets::CargoTargets::prepare(
        &root,
        &config,
        &[],
        &sources,
    ));
    let discovered = discover_tests_from_prepared_projects(
        &root,
        &config,
        TestRunner::Cargo,
        &prepared,
        &[],
        &tsconfig,
    )
    .unwrap();
    assert!(discovered.tests.is_empty());

    prepared
        .projects
        .insert(TestRunner::Vitest, Err("vitest config failed".to_string()));
    let error = discover_tests_from_prepared_projects(
        &root,
        &config,
        TestRunner::Vitest,
        &prepared,
        &[],
        &tsconfig,
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("vitest config failed"),
        "{error}"
    );

    let mut blocked = project("**/*.ts", "[");
    blocked.vitest_setup.push(setup(false, PathBuf::new()));
    prepared
        .projects
        .insert(TestRunner::Vitest, Ok(vec![blocked]));
    assert!(prepared.vitest_setup_projects().is_empty());
    assert!(prepared.project_filters().is_empty());

    let mut resolvable = project("**/*.ts", "");
    resolvable.vitest_setup.push(setup(false, PathBuf::new()));
    prepared
        .projects
        .insert(TestRunner::Vitest, Ok(vec![resolvable]));
    let catalog = TsConfigCatalog::forced(&root, TsConfig::default(), None);
    prepared.reresolve_vitest_setups(&root, &catalog, &[]);
    assert_eq!(
        prepared.projects[&TestRunner::Vitest].as_ref().unwrap()[0].vitest_setup[0].resolution_base,
        root
    );

    let mut reparse = project("**/*.ts", "");
    reparse.vitest_setup.push(setup(true, root.clone()));
    prepared
        .projects
        .insert(TestRunner::Vitest, Ok(vec![reparse]));
    let mut vitest = NoMistakesConfig::default();
    vitest.tests.vitest.configs = Some(StringOrList::One("missing.config.ts".to_string()));
    prepared.reparse_vitest_with_final_catalog(
        &root,
        &vitest,
        &[],
        Arc::new(catalog),
        Arc::new(sources),
    );
    let error = prepared
        .requested_runner_projects(TestRunner::Vitest)
        .unwrap_err();
    assert!(error.to_string().contains("does not exist"), "{error}");
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
        &root,
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
