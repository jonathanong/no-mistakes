use super::argument_rule_tests::check;

#[test]
fn exact_projection_keeps_finding_and_rejects_broader_prepared_reuse() {
    let source = crate::test_support::rule_fixture_root("forbidden-calls/argument-predicates");
    let saved = crate::test_support::materialize_saved_fixture(&source);
    let root = saved.path();
    let narrow_path = root.join("exact-projection.yml");
    let broader_path = root.join("exact-projection-broader.yml");
    let narrow = crate::config::v2::load_v2_config(root, Some(&narrow_path)).unwrap();
    let graph_plan = crate::codebase::rules::try_canonical_graph_plan(&narrow)
        .unwrap()
        .unwrap();
    let (mut graph, graph_context) =
        crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan(root, graph_plan);
    graph.function_calls |= super::source_only_eligible(&narrow);
    let mut plan = crate::codebase::check_facts::CheckFactPlan {
        graph,
        graph_context,
        ..Default::default()
    };
    plan.require_call_arguments(super::argument_projection(&narrow));
    let shared = crate::codebase::check_facts::collect_check_facts(
        root,
        crate::codebase::ts_source::discover_files(root, &[]),
        plan,
    );
    let findings =
        crate::codebase::rules::run_check_with_facts(root, Some(&narrow_path), None, &shared)
            .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 4);
    // Import-only supplemental facts overlap the richer primary entry. The
    // retained argument facts still cover this request after composition.
    let supplemental = crate::codebase::check_facts::collect_check_facts(
        root,
        vec![root.join("exact-projection.mjs")],
        crate::codebase::check_facts::CheckFactPlan {
            imports: true,
            ..Default::default()
        },
    );
    let composed = shared.fact_view_filling_missing(&supplemental);
    let composed_findings =
        crate::codebase::rules::run_check_with_facts(root, Some(&narrow_path), None, &composed)
            .unwrap();
    assert_eq!(composed_findings, findings);
    let error =
        crate::codebase::rules::run_check_with_facts(root, Some(&broader_path), None, &shared)
            .unwrap_err();
    assert!(
        error.to_string().contains("missing argument facts"),
        "{error:#}"
    );
}

#[test]
fn late_direct_eval_invalidates_retained_exact_argument_proofs() {
    let findings = check("exact-projection-eval.yml");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 3);
    assert!(findings[0].message.contains("unknown arguments"));
}

#[test]
fn synthetic_callback_keeps_ordinary_exact_call_finding() {
    // The callback edge participates in graph reachability but cannot satisfy
    // the rule's Call invocation filter; the later ordinary call still can.
    let findings = check("exact-projection-callback.yml");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 4);
    assert_eq!(findings[0].import.as_deref(), Some("execute"));
}

#[test]
fn malformed_target_options_precede_missing_argument_facts() {
    // A public prepared caller may provide full call sites without argument
    // values. Invalid YAML must still fail at option parsing, not demand.
    let source = crate::test_support::rule_fixture_root("forbidden-calls/argument-predicates");
    let saved = crate::test_support::materialize_saved_fixture(&source);
    let root = saved.path();
    let config_path = root.join("invalid-targets-scalar.yml");
    let config = crate::config::v2::load_v2_config(root, Some(&config_path)).unwrap();
    assert!(!super::argument_facts_required(&config));
    let graph_plan = crate::codebase::rules::try_canonical_graph_plan(&config)
        .unwrap()
        .unwrap();
    let (graph, graph_context) =
        crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan(root, graph_plan);
    let facts = crate::codebase::check_facts::collect_check_facts(
        root,
        crate::codebase::ts_source::discover_files(root, &[]),
        crate::codebase::check_facts::CheckFactPlan {
            graph,
            graph_context,
            ..Default::default()
        },
    );
    let error =
        crate::codebase::rules::run_check_with_facts(root, Some(&config_path), None, &facts)
            .unwrap_err();
    assert!(
        !error.to_string().contains("missing argument facts"),
        "{error:#}"
    );
    assert!(error.to_string().contains("targets"), "{error:#}");
}

#[test]
fn visible_non_indexable_file_root_retains_eager_error() {
    let source = crate::test_support::rule_fixture_root("forbidden-calls/argument-predicates");
    let saved = crate::test_support::materialize_saved_fixture(&source);
    let root = saved.path();
    let error_for = |name: &str| {
        crate::codebase::rules::run_check(root, Some(&root.join(name)), None)
            .unwrap_err()
            .to_string()
    };
    let source_only = error_for("non-indexable-root-exact.yml");
    let eager = error_for("non-indexable-root-eager.yml");
    assert_eq!(source_only, eager);
    assert!(eager.contains("no callable source"), "{eager}");
}

#[test]
fn precollected_fallback_facts_cannot_make_markdown_a_call_root() {
    use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactMap, TsFactPlan};
    let source = crate::test_support::rule_fixture_root("forbidden-calls/argument-predicates");
    let saved = crate::test_support::materialize_saved_fixture(&source);
    let root = saved.path();
    let source_path = root.join("exported-calls.mjs");
    let markdown = root.join("non-indexable-root.md");
    let plan = TsFactPlan {
        function_calls: true,
        ..Default::default()
    };
    let parsed = collect_ts_facts(std::slice::from_ref(&source_path), plan);
    let borrowed = parsed.get(&source_path).unwrap().clone();
    assert!(!borrowed.callable_scopes.is_empty());
    // A public precollected map can carry fallback facts for non-TS files.
    let facts = TsFactMap::from_iter_with_plan([(markdown.clone(), borrowed)], plan);
    let config =
        crate::config::v2::load_v2_config(root, Some(&root.join("non-indexable-root-exact.yml")))
            .unwrap();
    let application = config.rule_applications(super::RULE_ID)[0];
    let options: super::config::Options = application.try_rule_options().unwrap();
    let graph_files = [markdown.clone()];
    let graph_files = crate::codebase::check_facts::PathMembership::new(&graph_files);
    let error = super::source_only::file_roots(root, &options, &graph_files, &facts).unwrap_err();
    assert!(
        error.to_string().contains("no callable source"),
        "{error:#}"
    );
}

#[test]
fn exact_file_only_skips_canonical_calls_and_mixed_selector_restores_them() {
    use crate::codebase::dependencies::graph::call_collector_probe;
    let source = crate::test_support::rule_fixture_root("forbidden-calls/argument-predicates");
    let saved = crate::test_support::materialize_saved_fixture(&source);
    let root = saved.path();
    let plan_for = |name: &str| {
        let config = crate::config::v2::load_v2_config(root, Some(&root.join(name))).unwrap();
        crate::codebase::rules::try_canonical_graph_plan(&config)
            .unwrap()
            .unwrap()
    };
    assert!(!plan_for("exact-projection-noarg.yml").calls);
    assert!(plan_for("exact-projection-mixed.yml").calls);
    call_collector_probe::reset();
    let source_only = check("exact-projection-noarg.yml");
    assert_eq!(call_collector_probe::entries(), 0);
    assert_eq!(
        source_only.iter().map(|f| f.line).collect::<Vec<_>>(),
        vec![4, 6]
    );
    call_collector_probe::reset();
    let mixed = check("exact-projection-mixed.yml");
    assert!(call_collector_probe::entries() > 0);
    assert_eq!(mixed.len(), 4, "{mixed:?}");
    assert_eq!(
        mixed
            .iter()
            .filter(|f| f.message.contains("terminal"))
            .count(),
        2
    );
}

#[test]
fn unknown_exact_target_keeps_full_argument_and_graph_fallback() {
    let source = crate::test_support::rule_fixture_root("forbidden-calls/argument-predicates");
    let saved = crate::test_support::materialize_saved_fixture(&source);
    let root = saved.path();
    let config =
        crate::config::v2::load_v2_config(root, Some(&root.join("exact-unknown-broad.yml")))
            .unwrap();
    assert!(!super::source_only_eligible(&config));
    assert!(
        crate::codebase::rules::try_canonical_graph_plan(&config)
            .unwrap()
            .unwrap()
            .calls
    );
    assert!(!super::argument_projection(&config).unwrap().is_exact());
}

#[test]
fn missing_file_roots_cannot_claim_source_only_eligibility() {
    let root = crate::test_support::rule_fixture_root("forbidden-calls/argument-predicates");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join("exact-projection-no-roots.yml")))
            .unwrap();
    assert!(!super::source_only_eligible(&config));
}

#[test]
fn saved_function_scope_without_opaque_ids_keeps_source_call_finding() {
    use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
    let source = crate::test_support::rule_fixture_root("forbidden-calls/argument-predicates");
    let saved = crate::test_support::materialize_saved_fixture(&source);
    let root = saved.path();
    let path = root.join("exported-calls.mjs");
    let mut facts = collect_ts_facts(
        std::slice::from_ref(&path),
        TsFactPlan {
            function_calls: true,
            ..Default::default()
        },
    );
    let file = facts.get_mut(&path).unwrap();
    assert!(file.function_calls.iter().any(|call| call.caller.is_some()));
    file.callable_scope_ids.clear();
    for call in &mut file.function_calls {
        call.caller_id = None;
    }
    let config_path = root.join("exported-calls-exact.yml");
    let config = crate::config::v2::load_v2_config(root, Some(&config_path)).unwrap();
    let application = config.rule_applications(super::RULE_ID)[0];
    let options: super::config::Options = application.try_rule_options().unwrap();
    let graph_files = [path];
    let graph_files = crate::codebase::check_facts::PathMembership::new(&graph_files);
    let findings = super::source_only::check(
        root,
        &config,
        application,
        0,
        &options,
        &graph_files,
        &facts,
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 2);
}

#[test]
fn missing_prepared_source_calls_fail_instead_of_returning_no_findings() {
    use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactMap, TsFactPlan};
    let source = crate::test_support::rule_fixture_root("forbidden-calls/argument-predicates");
    let saved = crate::test_support::materialize_saved_fixture(&source);
    let root = saved.path();
    let path = root.join("exact-projection.mjs");
    let config =
        crate::config::v2::load_v2_config(root, Some(&root.join("exact-projection-noarg.yml")))
            .unwrap();
    let application = config.rule_applications(super::RULE_ID)[0];
    let options: super::config::Options = application.try_rule_options().unwrap();
    let graph_files = [path.clone()];
    let graph_files = crate::codebase::check_facts::PathMembership::new(&graph_files);
    let absent = TsFactMap::new();
    let error = super::source_only::check(
        root,
        &config,
        application,
        0,
        &options,
        &graph_files,
        &absent,
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("missing source calls"),
        "{error:#}"
    );

    let declared = TsFactMap::from_iter_with_plan(
        [],
        TsFactPlan {
            function_calls: true,
            ..Default::default()
        },
    );
    let error =
        super::source_only::file_roots(root, &options, &graph_files, &declared).unwrap_err();
    assert!(
        error.to_string().contains("missing for configured root"),
        "{error:#}"
    );

    let parsed = collect_ts_facts(
        &[path],
        TsFactPlan {
            function_calls: true,
            ..Default::default()
        },
    );
    let no_visible_paths: [std::path::PathBuf; 0] = [];
    let no_visible_paths = crate::codebase::check_facts::PathMembership::new(&no_visible_paths);
    let error =
        super::source_only::file_roots(root, &options, &no_visible_paths, &parsed).unwrap_err();
    assert!(
        error.to_string().contains("no callable source"),
        "{error:#}"
    );
}

#[test]
fn exact_file_roots_preserve_validation_and_deduplication() {
    let source = crate::test_support::rule_fixture_root("forbidden-calls/argument-predicates");
    let saved = crate::test_support::materialize_saved_fixture(&source);
    let root = saved.path();
    let error_for = |name: &str| {
        crate::codebase::rules::run_check(root, Some(&root.join(name)), None)
            .unwrap_err()
            .to_string()
    };
    assert!(error_for("exact-projection-excluded.yml").contains("excluded all configured roots"));
    assert!(error_for("exact-projection-missing.yml").contains("is not a file"));
    assert!(error_for("exact-projection-invalid-exclude.yml").contains("excludeRoots"));
    assert!(error_for("exact-projection-empty.yml").contains("no callable source"));
    assert!(error_for("exact-projection-parse-error.yml").contains("failed to parse"));
    assert_eq!(
        check("exact-projection-duplicate.yml"),
        check("exact-projection-noarg.yml")
    );
}
