use super::*;

#[test]
fn prepared_runner_and_playwright_configs_are_removed_after_overrides() {
    let root = root();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let visible = snapshot.paths_for(&root);
    assert!(visible.contains(&root.join("vitest.config.ts")));
    let sources = snapshot.source_store_for(&root);
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible_and_sources(
        None, &root, &visible, &sources,
    )
    .unwrap();
    let runner = Arc::new(crate::integration_tests::prepare_runner_configs(
        &root, &config, &visible, &tsconfig,
    ));
    assert!(runner.paths().any(|p| p.ends_with("vitest.config.ts")));
    let settings =
        crate::playwright::config::test_support::load_settings(&root, None, &[], None).unwrap();
    let mut playwright = PlaywrightFactPlan::default();
    playwright
        .add_source_settings(&root, settings, false, &snapshot)
        .unwrap();
    assert!(playwright
        .config_files()
        .contains(&root.join("playwright.config.ts")));
    let mut request = request(
        &root,
        &["web/primary.ts", "vitest.config.ts", "playwright.config.ts"],
        &[],
        CheckFactPlan {
            integration_runner_configs: Some(runner),
            ..Default::default()
        },
    );
    request.playwright = Some(playwright);
    let requests = [request];
    #[cfg(feature = "test-instrumentation")]
    super::historical::assert_parity(&requests);
    let mut slots = demands::graph_plans(&requests);
    let mut clones = 0;
    let demands = demands::demands_by_path::<true>(&requests, &mut slots, &mut clones);
    assert!(!demands.contains_key(&root.join("vitest.config.ts")));
    assert!(!demands.contains_key(&root.join("playwright.config.ts")));
    assert!(
        demands[&root.join("web/primary.ts")][0]
            .plan
            .as_ref()
            .graph
            .imports
    );
}
