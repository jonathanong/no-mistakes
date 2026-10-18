#[test]
fn aggregate_prepared_domains_do_not_reload_the_unified_config() {
    let aggregate = include_str!("../prepared.rs");
    let playwright = concat!(
        include_str!("../../playwright/rules/prepared_entrypoints.rs"),
        include_str!("../../playwright/rules/prepared.rs"),
    );
    let graph = include_str!("../../codebase/dependencies/graph/files_config_prepared.rs");
    let rules = concat!(
        include_str!("../../codebase/rules/run/prepared.rs"),
        include_str!("../../codebase/rules/run/prepared/execution.rs"),
        include_str!("../../codebase/rules/run/prepared/execution/helpers.rs"),
    );

    // Aggregate preparation must consume the session-owned manifest once and pass the loaded
    // value onward; direct loading here would split the cache from other request consumers.
    assert_eq!(
        aggregate
            .matches("session.config_with_path(root, config_path)?")
            .count(),
        1
    );
    assert!(!aggregate.contains("load_v2_config_from_visible"));
    assert!(!aggregate.contains("prepare_check_from_visible"));
    let aggregate_playwright = playwright
        .split("pub fn prepare_from_snapshot")
        .nth(1)
        .and_then(|source| source.split("fn prepare_with_settings").next())
        .expect("aggregate Playwright preparation body");
    assert!(aggregate_playwright.contains("settings_from_loaded_v2"));
    assert!(!aggregate_playwright.contains("load_settings_from_visible"));
    assert!(playwright.contains("load_settings_from_visible"));
    assert!(playwright.contains("prepared_selections"));
    assert!(graph.contains("settings_from_loaded_v2"));
    assert!(!graph.contains("load_settings_from_visible"));
    let aggregate_rules = rules
        .split("pub fn run_check_with_config_and_facts_and_playwright")
        .nth(1)
        .expect("aggregate rules entrypoint body");
    assert!(aggregate_rules.contains("require_storybook_stories::check_with_prepared_facts"));
    assert!(!aggregate_rules.contains("require_storybook_stories::check_with_facts("));
}

#[test]
fn aggregate_react_filters_the_shared_snapshot_without_rediscovery() {
    let react = include_str!("../../react_traits/pipeline/run_with_facts.rs");

    assert!(react.contains("shared.files()"));
    assert!(react.contains("expand_globs_from_files"));
    assert!(!react.contains("discover_visible_paths"));
    assert!(!react.contains("expand_globs(root"));
}

#[test]
fn aggregate_storybook_reuses_the_scoped_tsconfig_catalog() {
    let prepared = include_str!("../../codebase/rules/require_storybook_stories/prepared.rs");

    assert!(prepared.contains("ScopedImportResolver::new_in_session"));
    assert!(prepared.contains("prepared_tsconfig_catalog"));
    assert!(prepared.contains(".files()"));
    assert_eq!(
        prepared
            .matches("ScopedImportResolver::new_in_session")
            .count(),
        1
    );
}

#[test]
fn aggregate_rule_coordinator_delegates_variant_dispatch() {
    let execution = include_str!("../../codebase/rules/run/prepared/execution.rs");
    let helpers = include_str!("../../codebase/rules/run/prepared/execution/helpers.rs");
    let independent_collect =
        include_str!("../../codebase/rules/run/prepared/execution/independent_collect.rs");
    let coordinator = execution
        .split("pub(super) fn run")
        .nth(1)
        .expect("prepared rule coordinator");
    let storybook_block = independent_collect
        .split("pub(super) fn storybook(")
        .nth(1)
        .and_then(|source| source.split("pub(super) fn playwright(").next())
        .expect("Storybook independent collector");

    // Keep per-rule variant selection out of the aggregate coordinator so its
    // complexity remains bounded as additional rules are introduced.
    assert!(execution.contains("mod helpers;"));
    assert!(execution.contains("mod independent;"));
    assert!(coordinator.contains("independent::collect"));
    assert!(!coordinator.contains("REQUIRE_STORYBOOK_STORIES"));
    assert!(!coordinator.contains("storybook_findings"));
    assert!(helpers.contains("pub(super) fn storybook_findings("));
    assert!(helpers.contains("check_with_prepared_facts_for_aggregate"));
    assert_eq!(storybook_block.matches("storybook_findings(").count(), 1);
    assert!(storybook_block.contains("prepared_tsconfig_catalog"));
    assert!(!storybook_block.contains("prepared_tsconfig,"));
    assert!(!storybook_block.contains("check_with_prepared_facts_and_inferred"));
}
