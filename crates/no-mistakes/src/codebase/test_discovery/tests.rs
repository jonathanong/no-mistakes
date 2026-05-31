use super::*;

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
