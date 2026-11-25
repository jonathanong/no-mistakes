use super::RULE_ID;

#[test]
fn workspace_relative_imports_reuse_the_rust_check_graph_and_existing_facts() {
    use crate::codebase::check_facts::{collect_check_facts, CheckFactPlan};
    use crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan;
    use crate::codebase::rules::{run_check, run_check_with_facts, try_canonical_graph_plan};
    use crate::codebase::ts_source::discover_files;
    use crate::config::v2::load_v2_config;

    let root = crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root(RULE_ID),
    );
    let config = load_v2_config(&root, None).unwrap();
    let plan = try_canonical_graph_plan(&config).unwrap().unwrap();
    let (graph, graph_context) = ts_fact_plan_and_context_for_plan(&root, plan);
    let facts = collect_check_facts(
        &root,
        discover_files(&root, &[]),
        CheckFactPlan {
            graph,
            graph_context,
            ..Default::default()
        },
    );
    let standalone = run_check(&root, None, None).unwrap();
    assert_eq!(standalone.len(), 7);
    assert_eq!(
        run_check_with_facts(&root, None, None, &facts).unwrap(),
        standalone
    );
}

#[test]
fn workspace_relative_imports_reports_malformed_selected_source_to_rust_callers() {
    use crate::codebase::rules::run_check;

    let root = crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root(RULE_ID),
    );
    let config = root.join("incomplete.yml");
    let error = run_check(&root, Some(&config), None).unwrap_err();
    let failure = error
        .downcast_ref::<crate::codebase::check_outcome::CheckInputFailure>()
        .unwrap();
    assert!(failure
        .file
        .as_deref()
        .is_some_and(|file| file.ends_with("packages/app/src/gaps/broken.mts")));
}
