use super::*;

#[test]
fn aggregate_check_injects_prepared_config_into_every_domain() {
    let runner = concat!(
        include_str!("../../check_runner.rs"),
        include_str!("../run_all.rs"),
    );
    // The independent Rayon branch lives beside this entrypoint. Count its calls too.
    let prepared = concat!(
        include_str!("../prepared.rs"),
        include_str!("../prepared/independent.rs"),
    );
    let graph_plan = include_str!("../graph_plan.rs");
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
        "playwright_projects",
    ] {
        assert!(
            parallel.contains(prepared_input),
            "domain dispatcher must receive {prepared_input}"
        );
    }

    for shared_entrypoint in [
        "run_check_with_prepared_facts_for_aggregate",
        "run_check_with_declared_plan_and_suppression",
        "queue::analyze_project_with_prepared_facts_and_catalog_and_session",
        "integration_tests::check_with_prepared_facts_catalog_and_session",
        "unique_exports::analyze_project_with_prepared_facts_catalog_and_inferred_and_session_for_check",
        "run_filesystem_rules_with_config_snapshot_catalog_sources_and_facts",
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
            .matches("session.config_with_path(root, config_path)?")
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
    assert!(include_str!("../fact_collection.rs").contains("clear_request_parse_cache"));
    assert!(include_str!("../fact_collection.rs").contains("parse.files_after_extract"));
    assert!(graph_plan.contains("prepare_graph_config_with_test_filter"));
    assert!(graph_plan.contains("ts_fact_plan_and_context_for_plan_with_prepared"));
    assert!(!runner.contains("react_traits::check_enabled"));
    assert!(prepared.contains("prepare_from_snapshot_with_catalog"));
    assert!(!tasks.contains("queue::analyze_project_with_prepared_facts("));
    assert!(!tasks.contains("integration_tests::check_with_prepared_facts("));
    assert!(!tasks.contains("unique_exports::analyze_project_with_prepared_facts("));
    assert!(!tasks.contains("load_v2_config"));
    assert!(!tasks.contains("discover_visible_paths"));
}

#[test]
fn aggregate_framework_root_inference_reuses_precomputed_visible_roots() {
    let prepared = concat!(
        include_str!("../prepared.rs"),
        include_str!("../prepared/independent.rs"),
    );
    let runner = concat!(
        include_str!("../../check_runner.rs"),
        include_str!("../run_all.rs"),
    );
    let discovery = include_str!("../../check_discovery.rs");
    let rules = concat!(
        include_str!("../../codebase/rules/run/prepared.rs"),
        include_str!("../../codebase/rules/run/prepared/execution.rs"),
        include_str!("../../codebase/rules/run/prepared/execution/helpers.rs"),
        include_str!("../../codebase/rules/run/prepared/execution/independent.rs"),
        include_str!("../../codebase/rules/run/prepared/execution/independent_collect.rs"),
    );
    let rule_mod = include_str!("../../codebase/rules/mod.rs");
    let rule_reexports = include_str!("../../codebase/rules/reexports.rs");
    let unique_exports = include_str!("../../codebase/unique_exports/with_facts/prepared.rs");
    let tasks = check_task_sources();

    assert_eq!(prepared.matches("InferredRoots::from_visible").count(), 1);
    assert!(runner.contains("&prepared.inferred_roots"));
    assert!(discovery.contains("unique_exports_project_roots_with_inferred"));
    assert!(
        rules.contains("req.inferred_roots"),
        "independent rule bodies must reuse the precomputed inferred_roots option"
    );
    assert!(rule_reexports.contains("target_roots_with_inferred"));
    assert!(unique_exports.contains("project_roots_for_rule_with_inferred"));

    for source in [
        discovery,
        rules,
        rule_mod,
        rule_reexports,
        unique_exports,
        tasks.as_str(),
    ] {
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
