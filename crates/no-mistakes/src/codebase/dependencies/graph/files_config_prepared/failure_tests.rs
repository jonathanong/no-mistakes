use super::*;

#[test]
fn graph_failure_preserves_unrelated_project_and_app_consumers() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/playwright/partial-config-failure"),
    );
    let config: crate::config::v2::NoMistakesConfig = serde_yaml::from_str(include_str!(
        "../../../../../../../fixtures/playwright/partial-config-failure/graph-scopes.yml"
    ))
    .unwrap();
    let files = [root.join("good/page.tsx"), root.join("broken/page.tsx")];
    let failure = PlaywrightPreparationFailure {
        file: Some(root.join("playwright.projects.config.ts")),
        project: Some("broken".into()),
        config_name: None,
        message: "expected string literal for testDir".into(),
        roots: vec![root.join("broken")],
    };
    assert!(!failure
        .affects_rule(&root, &config, &config.rules[0], &files)
        .unwrap());
    assert!(failure
        .affects_rule(&root, &config, &config.rules[1], &files)
        .unwrap());
    let mut app_only = config.rules[0].clone();
    app_only.tests.playwright.clear();
    assert!(!failure
        .affects_rule(&root, &config, &app_only, &files)
        .unwrap());
    let mut shared_failure = failure;
    shared_failure.project = None;
    shared_failure.roots = vec![root.join("good"), root.join("broken")];
    assert!(shared_failure
        .affects_rule(&root, &config, &config.rules[0], &files)
        .unwrap());
}

fn partial_graph_fixture() -> (
    PathBuf,
    crate::codebase::ts_source::VisiblePathSnapshot,
    PreparedGraphConfig,
) {
    let root = crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/playwright/partial-config-failure"),
    );
    let files = [
        "playwright.good.config.ts",
        "playwright.shared.config.ts",
        "playwright.projects.config.ts",
        "good/page.tsx",
        "tests/good.spec.ts",
    ]
    .map(|path| root.join(path));
    let visible = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(&root, &files);
    let config = crate::config::v2::NoMistakesConfig::default();
    let mut prepared = prepare_graph_config(
        &root,
        GraphBuildPlan::default(),
        &Default::default(),
        &config,
        &visible,
    )
    .unwrap();
    prepared.playwright_settings.push(
        crate::playwright::config::settings_from_loaded_v2(
            &root,
            &config,
            &[],
            None,
            None,
            &visible,
        )
        .unwrap(),
    );
    (root, visible, prepared)
}

#[test]
fn failed_shared_playwright_config_disables_only_playwright_edges() {
    let (root, visible, mut prepared) = partial_graph_fixture();
    prepared.playwright_settings[0].playwright_configs = vec!["playwright.shared.config.ts".into()];
    let (facts, failures) =
        prepared.playwright_fact_plan_partial(&root, &Default::default(), &visible);
    assert!(facts.is_none());
    assert_eq!(failures.len(), 1);
    assert_eq!(
        failures[0].file.as_deref(),
        Some(root.join("playwright.shared.config.ts").as_path())
    );
    assert!(failures[0].message.contains("testDir"));
    let plan = prepared.effective_plan(GraphBuildPlan {
        imports: true,
        playwright_routes: true,
        playwright_selectors: true,
        ..Default::default()
    });
    assert!(plan.imports);
    assert!(!plan.playwright_routes && !plan.playwright_selectors);
}

#[test]
fn partial_project_failure_retains_healthy_playwright_fact_scope() {
    let (root, visible, mut prepared) = partial_graph_fixture();
    prepared.playwright_settings[0].playwright_configs =
        vec!["playwright.projects.config.ts".into()];
    let (facts, failures) =
        prepared.playwright_fact_plan_partial(&root, &Default::default(), &visible);
    assert!(facts.is_some());
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].project.as_deref(), Some("broken"));
    assert!(failures[0].message.contains("testDir"));
    assert!(!prepared.playwright_failed);
    assert_eq!(prepared.playwright_settings.len(), 1);
}

#[test]
fn missing_named_config_is_an_owned_preparation_failure() {
    let (root, visible, mut prepared) = partial_graph_fixture();
    prepared.playwright_settings[0].playwright_configs = vec!["playwright.good.config.ts".into()];
    prepared.playwright_settings[0].project = Some("absent".into());
    let (facts, failures) =
        prepared.playwright_fact_plan_partial(&root, &Default::default(), &visible);
    assert!(facts.is_none());
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].project.as_deref(), Some("absent"));
    assert!(failures[0].file.is_none());
    assert!(failures[0].message.contains("absent"));
    assert!(prepared.playwright_failed);
}
