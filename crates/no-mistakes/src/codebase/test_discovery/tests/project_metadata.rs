use super::*;

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
        workspace: false,
        policy_name: Some("browser".to_string()),
        runner_project_arg: Some("browser".to_string()),
        scope: None,
        include: vec!["src/utils.mts".to_string()],
        exclude: Vec::new(),
        declared_deadlines: Default::default(),
        matcher_warnings: Vec::new(),
        vitest_concurrent: false,
        vitest_setup: Vec::new(),
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
        workspace: false,
        policy_name: Some("top-level-config-name".to_string()),
        runner_project_arg: None,
        scope: None,
        include: vec!["src/utils.mts".to_string()],
        exclude: Vec::new(),
        declared_deadlines: Default::default(),
        matcher_warnings: Vec::new(),
        vitest_concurrent: false,
        vitest_setup: Vec::new(),
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
