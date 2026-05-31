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
fn vitest_project_discovery_without_playwright_projects_keeps_matching_tests() {
    let root = fixture_root("symbols-output");
    let config = NoMistakesConfig::default();
    let projects = vec![ConfigProject {
        config: Some("vitest.config.mts".to_string()),
        name: Some("all-specs".to_string()),
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
        name: Some("browser".to_string()),
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
        .find(|project| project.name.as_deref() == Some("dynamic"))
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
        .find(|project| project.name.as_deref() == Some("dynamic"))
        .unwrap();
    assert_eq!(project.config, None);
}
