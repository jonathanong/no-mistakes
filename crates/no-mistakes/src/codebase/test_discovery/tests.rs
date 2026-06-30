use super::*;
use crate::config::v2::schema::{StringOrList, TestProjectPolicy};
use std::collections::BTreeMap;

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis")
            .join(name)
            .join("fixture"),
    )
}

#[test]
fn dotnet_strict_project_discovery_errors_on_missing_projects() {
    let root = fixture_root("dotnet-test-plan");
    let mut config = crate::config::v2::load_v2_config(&root, None).unwrap();
    config.tests.dotnet.projects.insert(
        "missing".to_string(),
        crate::config::v2::schema::DotnetProjectConfig {
            project: "dotnet-clients/tests/Missing/Missing.csproj".to_string(),
            include: Vec::new(),
            exclude: Vec::new(),
            test: true,
        },
    );

    let error = projects::runner_projects(&root, &config, TestRunner::Dotnet).unwrap_err();
    assert!(error
        .to_string()
        .contains("configured dotnet project `missing`"));
}

#[test]
fn dotnet_lossy_project_discovery_skips_missing_projects() {
    let root = fixture_root("dotnet-test-plan");
    let mut config = crate::config::v2::load_v2_config(&root, None).unwrap();
    config.tests.dotnet.projects.insert(
        "missing".to_string(),
        crate::config::v2::schema::DotnetProjectConfig {
            project: "dotnet-clients/tests/Missing/Missing.csproj".to_string(),
            include: Vec::new(),
            exclude: Vec::new(),
            test: true,
        },
    );

    let projects = projects::runner_projects_lossy(&root, &config, TestRunner::Dotnet);
    assert!(projects
        .iter()
        .any(|project| project.policy_name.as_deref() == Some("app-tests")));
    assert!(!projects
        .iter()
        .any(|project| project.policy_name.as_deref() == Some("missing")));
}

#[test]
fn dotnet_project_discovery_honors_include_override() {
    let root = fixture_root("dotnet-test-plan");
    let mut config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let project = config
        .tests
        .dotnet
        .projects
        .get_mut("app-tests")
        .expect("fixture should define app-tests");
    project.include = vec!["dotnet-clients/tests/App.Tests/ParserEdgeCases.cs".to_string()];

    let projects = projects::runner_projects(&root, &config, TestRunner::Dotnet).unwrap();
    let app_tests = projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("app-tests"))
        .expect("app-tests project should be discovered");

    assert_eq!(
        app_tests.include,
        vec!["dotnet-clients/tests/App.Tests/ParserEdgeCases.cs"]
    );
}

#[test]
fn dotnet_project_discovery_falls_back_when_no_xunit_files_are_known() {
    let root = fixture_root("dotnet-test-plan");
    let mut config = crate::config::v2::load_v2_config(&root, None).unwrap();
    config.tests.dotnet.solutions.clear();
    config.tests.dotnet.projects.clear();
    config.tests.dotnet.projects.insert(
        "fallback".to_string(),
        crate::config::v2::schema::DotnetProjectConfig {
            project: "dotnet-clients/src/Fallback/Fallback.csproj".to_string(),
            include: Vec::new(),
            exclude: Vec::new(),
            test: true,
        },
    );

    let projects = projects::runner_projects(&root, &config, TestRunner::Dotnet).unwrap();

    assert_eq!(projects.len(), 1);
    assert_eq!(
        projects[0].include,
        vec!["dotnet-clients/src/Fallback/**/*.cs"]
    );
}

#[test]
fn test_runner_framework_maps_dotnet_and_swift() {
    assert_eq!(
        TestRunner::Dotnet.framework(),
        crate::integration_tests::types::Framework::Dotnet
    );
    assert_eq!(
        TestRunner::Swift.framework(),
        crate::integration_tests::types::Framework::Swift
    );
}

#[test]
fn vitest_project_discovery_without_playwright_projects_keeps_matching_tests() {
    let root = fixture_root("symbols-output");
    let config = NoMistakesConfig::default();
    let projects = vec![ConfigProject {
        config: Some("vitest.config.mts".to_string()),
        policy_name: Some("all-specs".to_string()),
        runner_project_arg: Some("all-specs".to_string()),
        scope: None,
        include: vec!["src/utils.mts".to_string()],
        exclude: Vec::new(),
    }];

    let discovered = discover_from_projects(&root, &config, TestRunner::Vitest, projects).unwrap();

    let rel_tests: Vec<String> = discovered
        .tests
        .iter()
        .map(|path| crate::codebase::ts_source::relative_slash_path(&root, path))
        .collect();
    assert_eq!(rel_tests, vec!["src/utils.mts"]);
}

#[test]
fn vitest_explicit_project_matches_playwright_owned_file() {
    let root = fixture_root("symbols-output");
    let mut config = NoMistakesConfig::default();
    config.tests.playwright.projects.insert(
        "chromium".to_string(),
        TestProjectPolicy {
            include: vec!["src/utils.mts".to_string()],
            ..Default::default()
        },
    );
    let projects = vec![ConfigProject {
        config: Some("vitest.config.mts".to_string()),
        policy_name: Some("browser".to_string()),
        runner_project_arg: Some("browser".to_string()),
        scope: None,
        include: vec!["src/utils.mts".to_string()],
        exclude: Vec::new(),
    }];

    let discovered = discover_from_projects(&root, &config, TestRunner::Vitest, projects).unwrap();

    let rel_tests: Vec<String> = discovered
        .tests
        .iter()
        .map(|path| crate::codebase::ts_source::relative_slash_path(&root, path))
        .collect();
    assert_eq!(rel_tests, vec!["src/utils.mts"]);
}

#[test]
fn target_metadata_uses_executable_project_name_only() {
    let root = fixture_root("symbols-output");
    let config = NoMistakesConfig::default();
    let projects = vec![ConfigProject {
        config: Some("playwright.config.ts".to_string()),
        policy_name: Some("top-level-config-name".to_string()),
        runner_project_arg: None,
        scope: None,
        include: vec!["src/utils.mts".to_string()],
        exclude: Vec::new(),
    }];

    let discovered =
        discover_from_projects(&root, &config, TestRunner::Playwright, projects).unwrap();
    let target = discovered
        .targets_by_path
        .values()
        .next()
        .unwrap()
        .first()
        .unwrap();

    assert_eq!(target.project, None);
    assert!(!target.runner_args.contains(&"--project".to_string()));
}

#[test]
fn policy_only_project_inherits_single_explicit_runner_config() {
    let root = fixture_root("symbols-output");
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(StringOrList::One(
        "configs/vitest.workspace.mts".to_string(),
    ));
    config.tests.vitest.projects.insert(
        "dynamic".to_string(),
        TestProjectPolicy {
            include: vec!["src/utils.mts".to_string()],
            integration_suites: BTreeMap::from([("openai".to_string(), Vec::new())]),
            ..Default::default()
        },
    );

    let projects = projects::runner_projects_lossy(&root, &config, TestRunner::Vitest);

    let project = projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("dynamic"))
        .unwrap();
    assert_eq!(
        project.config.as_deref(),
        Some("configs/vitest.workspace.mts")
    );
}

#[test]
fn policy_only_project_does_not_guess_from_multiple_explicit_configs() {
    let root = fixture_root("symbols-output");
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(StringOrList::Many(vec![
        "configs/vitest.workspace.mts".to_string(),
        "configs/vitest.browser.mts".to_string(),
    ]));
    config.tests.vitest.projects.insert(
        "dynamic".to_string(),
        TestProjectPolicy {
            include: vec!["src/utils.mts".to_string()],
            integration_suites: BTreeMap::from([("openai".to_string(), Vec::new())]),
            ..Default::default()
        },
    );

    let projects = projects::runner_projects_lossy(&root, &config, TestRunner::Vitest);

    let project = projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("dynamic"))
        .unwrap();
    assert_eq!(project.config, None);
}

#[test]
fn policy_only_project_discovery_preserves_fallback_tests_outside_policy() {
    let root = fixture_root("test-discovery-policy-fallback");
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.projects.insert(
        "policy".to_string(),
        TestProjectPolicy {
            include: vec!["src/policy.test.mts".to_string()],
            integration_suites: BTreeMap::from([("openai".to_string(), Vec::new())]),
            ..Default::default()
        },
    );

    let discovered = discover_tests(&root, &config, TestRunner::Vitest).unwrap();

    let rel_tests: Vec<String> = discovered
        .tests
        .iter()
        .map(|path| crate::codebase::ts_source::relative_slash_path(&root, path))
        .collect();
    assert!(rel_tests.contains(&"src/policy.test.mts".to_string()));
    assert!(rel_tests.contains(&"src/fallback.test.mts".to_string()));
}

#[test]
fn vitest_config_without_include_uses_globset_compatible_defaults() {
    let root = fixture_root("test-discovery-vitest-defaults");
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(StringOrList::One("vitest.config.mts".to_string()));

    let discovered = discover_tests(&root, &config, TestRunner::Vitest).unwrap();

    let rel_tests: Vec<String> = discovered
        .tests
        .iter()
        .map(|path| crate::codebase::ts_source::relative_slash_path(&root, path))
        .collect();
    assert_eq!(rel_tests, vec!["src/default.test.ts"]);
    assert!(!discovered.used_fallback);
}

#[test]
fn vitest_fallback_skips_playwright_policy_tests() {
    let root = fixture_root("test-discovery-policy-fallback");
    let mut config = NoMistakesConfig::default();
    config.tests.playwright.projects.insert(
        "chromium".to_string(),
        TestProjectPolicy {
            include: vec!["e2e/**/*.spec.ts".to_string()],
            ..Default::default()
        },
    );

    let discovered = discover_tests(&root, &config, TestRunner::Vitest).unwrap();

    let rel_tests: Vec<String> = discovered
        .tests
        .iter()
        .map(|path| crate::codebase::ts_source::relative_slash_path(&root, path))
        .collect();
    assert!(rel_tests.contains(&"src/fallback.test.mts".to_string()));
    assert!(!rel_tests.contains(&"e2e/home.spec.ts".to_string()));
}

#[test]
fn playwright_policy_exclude_prevents_generic_fallback() {
    let root = fixture_root("test-discovery-policy-fallback");
    let mut config = NoMistakesConfig::default();
    config.tests.playwright.projects.insert(
        "chromium".to_string(),
        TestProjectPolicy {
            include: vec!["e2e/**/*.spec.ts".to_string()],
            exclude: vec!["e2e/flaky.spec.ts".to_string()],
            ..Default::default()
        },
    );

    let discovered = discover_tests(&root, &config, TestRunner::Playwright).unwrap();

    let rel_tests: Vec<String> = discovered
        .tests
        .iter()
        .map(|path| crate::codebase::ts_source::relative_slash_path(&root, path))
        .collect();
    assert!(rel_tests.contains(&"e2e/home.spec.ts".to_string()));
    assert!(!rel_tests.contains(&"e2e/flaky.spec.ts".to_string()));
}

#[test]
fn playwright_fallback_skips_helpers_in_e2e_directories() {
    let root = fixture_root("test-discovery-policy-fallback");
    let config = NoMistakesConfig::default();

    let discovered = discover_tests(&root, &config, TestRunner::Playwright).unwrap();

    let rel_tests: Vec<String> = discovered
        .tests
        .iter()
        .map(|path| crate::codebase::ts_source::relative_slash_path(&root, path))
        .collect();
    assert!(rel_tests.contains(&"tests/e2e/home.spec.ts".to_string()));
    assert!(!rel_tests.contains(&"tests/e2e/helpers.ts".to_string()));
}
