#[test]
fn aggregate_check_injects_prepared_config_into_every_domain() {
    let runner = include_str!("../../check_runner.rs");
    let prepared = include_str!("../prepared.rs");
    let forbidden_plan = include_str!("../forbidden_plan.rs");
    let parallel = include_str!("../../check_parallel.rs");
    let tasks = check_task_sources();

    for prepared_input in [
        "prepared_react",
        "prepared_playwright",
        "prepared_graph",
        "prepared_tsconfig",
        "visible_paths",
        "inferred_roots",
        "codebase_config",
        "vitest_projects",
    ] {
        assert!(
            parallel.contains(prepared_input),
            "domain dispatcher must receive {prepared_input}"
        );
    }

    for shared_entrypoint in [
        "run_check_with_prepared_facts",
        "run_check_with_config_facts_playwright_and_graph",
        "check_with_prepared_facts",
        "analyze_project_with_prepared_facts",
        "run_filesystem_rules_with_config_snapshot_catalog_and_sources",
    ] {
        assert!(
            tasks.contains(shared_entrypoint),
            "aggregate task must call {shared_entrypoint}"
        );
    }

    assert!(prepared.contains("prepare_check_from_loaded_config"));
    // The session is the canonical manifest cache boundary. Reintroducing a direct resolver here
    // would bypass request-wide config/tsconfig reuse even though it looks locally self-contained.
    assert_eq!(
        prepared
            .matches("session.config(root, config_path)?")
            .count(),
        1
    );
    assert_eq!(
        prepared
            .matches("session.tsconfig(root, tsconfig_path)?")
            .count(),
        1
    );
    assert!(!prepared.contains("resolve_tsconfig_from_visible"));
    assert!(forbidden_plan.contains("prepare_graph_config"));
    assert!(forbidden_plan.contains("ts_fact_plan_and_context_for_plan_with_prepared"));
    assert!(!runner.contains("react_traits::check_enabled"));
    assert!(tasks.contains("queue::analyze_project_with_prepared_facts"));
    assert!(!tasks.contains("queue::analyze_project_with_facts"));
    assert!(!tasks.contains("load_v2_config"));
    assert!(!tasks.contains("discover_visible_paths"));
}

#[test]
fn aggregate_framework_root_inference_reuses_precomputed_visible_roots() {
    let prepared = include_str!("../prepared.rs");
    let runner = include_str!("../../check_runner.rs");
    let discovery = include_str!("../../check_discovery.rs");
    let rules = concat!(
        include_str!("../../codebase/rules/run/prepared.rs"),
        include_str!("../../codebase/rules/run/prepared/execution.rs"),
        include_str!("../../codebase/rules/run/prepared/execution/helpers.rs"),
    );
    let rule_roots = include_str!("../../codebase/rules/mod.rs");
    let unique_exports = include_str!("../../codebase/unique_exports/with_facts/prepared.rs");
    let tasks = check_task_sources();

    assert_eq!(prepared.matches("InferredRoots::from_visible").count(), 1);
    assert!(runner.contains("&prepared.inferred_roots"));
    assert!(discovery.contains("unique_exports_project_roots_with_inferred"));
    assert!(rules.contains("Some(inferred_roots)"));
    assert!(rule_roots.contains("target_roots_with_inferred"));
    assert!(unique_exports.contains("project_roots_for_rule_with_inferred"));
    assert!(tasks.contains("analyze_project_with_prepared_facts_and_inferred"));

    for source in [discovery, rules, rule_roots, unique_exports, tasks.as_str()] {
        for discovery_wrapper in [
            "infer_nextjs_root(",
            "infer_remix_root(",
            "infer_vitejs_root(",
        ] {
            assert!(
                !source.contains(discovery_wrapper),
                "aggregate path must not call {discovery_wrapper}"
            );
        }
    }
}

#[test]
fn aggregate_vitest_ci_coverage_reuses_the_request_snapshot() {
    let prepared = include_str!("../prepared.rs");
    let tasks = check_task_sources();
    let dispatcher = include_str!("../../codebase/rules/filesystem_dispatch.rs");
    let catalog = include_str!("../../codebase/rules/vitest_project_catalog.rs");
    let mapping = include_str!("../../codebase/rules/vitest_project_mapping/project_sources.rs");
    let coverage = include_str!("../../codebase/rules/vitest_ci_path_coverage/projects.rs");
    let workflows =
        include_str!("../../codebase/rules/vitest_ci_path_coverage/workflow_filters.rs");

    assert_eq!(
        prepared.matches("prepare_vitest_project_catalog(").count(),
        1
    );
    assert!(tasks.contains("run_filesystem_rules_with_config_snapshot_catalog_and_sources"));
    assert!(dispatcher.contains("check_with_files_and_catalog"));
    assert!(dispatcher.contains("check_with_files_from_snapshot_catalog_and_sources"));
    assert_eq!(catalog.matches("load_projects_from_visible(").count(), 1);
    assert!(!mapping.contains("VisiblePathSnapshot::new"));
    assert!(!coverage.contains("VisiblePathSnapshot::new"));
    let aggregate_mapping = mapping
        .split("Some(catalog) => catalog.config_projects()?")
        .nth(1)
        .and_then(|source| source.split("None =>").next())
        .expect("mapping prepared-catalog branch");
    assert!(!aggregate_mapping.contains("load_projects("));
    let aggregate_coverage = coverage
        .split("Some(catalog) => catalog.config_projects()?")
        .nth(1)
        .and_then(|source| source.split("None =>").next())
        .expect("coverage prepared-catalog branch");
    assert!(!aggregate_coverage.contains("load_projects("));
    assert!(workflows.contains("discover_workflow_files_from_snapshot"));
    assert!(!workflows.contains("discover_workflow_files(root"));
}

fn check_task_sources() -> String {
    // Architecture assertions must cover the complete production task module,
    // including helpers split out to keep each Rust source under the size gate.
    [
        include_str!("../../check_tasks.rs"),
        include_str!("../../check_tasks/filesystem.rs"),
    ]
    .concat()
}

#[test]
fn aggregate_prepared_domains_do_not_reload_the_unified_config() {
    let aggregate = include_str!("../prepared.rs");
    let playwright = include_str!("../../playwright/rules/prepared.rs");
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
            .matches("session.config(root, config_path)?")
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
fn aggregate_storybook_prepares_visible_tsconfig_per_project_root() {
    let prepared = include_str!("../../codebase/rules/require_storybook_stories/prepared.rs");

    assert!(prepared.contains("explicit_tsconfig_path.is_some()"));
    assert!(prepared.contains("automatic_tsconfigs.get(project_root)"));
    assert!(prepared.contains("resolve_tsconfig_from_visible"));
    assert!(prepared.contains("shared.files()"));
}

#[test]
fn aggregate_rule_coordinator_delegates_variant_dispatch() {
    let execution = include_str!("../../codebase/rules/run/prepared/execution.rs");
    let helpers = include_str!("../../codebase/rules/run/prepared/execution/helpers.rs");
    let coordinator = execution
        .split("pub(super) fn run")
        .nth(1)
        .expect("prepared rule coordinator");
    let storybook_block = coordinator
        .split("if rule_enabled(config, REQUIRE_STORYBOOK_STORIES)")
        .nth(1)
        .and_then(|source| {
            source
                .split("if crate::playwright::rules::configured")
                .next()
        })
        .expect("Storybook coordinator block");

    // Keep per-rule variant selection out of the aggregate coordinator so its
    // complexity remains bounded as additional rules are introduced.
    assert!(execution.contains("mod helpers;"));
    assert!(execution.contains("use helpers::{storybook_findings, suppress_findings};"));
    assert!(helpers.contains("pub(super) fn storybook_findings("));
    assert!(helpers.contains("check_with_prepared_facts_and_inferred_and_session"));
    assert_eq!(storybook_block.matches("storybook_findings(").count(), 1);
    assert!(!storybook_block.contains("check_with_prepared_facts_and_inferred"));
}
