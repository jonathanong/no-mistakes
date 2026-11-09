use super::*;
use crate::codebase::ts_source::facts::collect_ts_facts;

#[test]
fn native_callback_parameters_keep_all_known_callers_without_changing_calls() {
    let root =
        crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-storage-wrapper");
    let files = GraphFiles::discover(&root);
    let plan = GraphBuildPlan {
        calls: true,
        assertion_owners: true,
        ..Default::default()
    };
    let facts = collect_ts_facts(files.indexable(), plan.ts_fact_plan());
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let build = |plan| {
        DepGraph::build_with_plan_files_config_and_facts(
            &root,
            &tsconfig,
            plan,
            &files,
            None,
            Some(&facts),
        )
        .unwrap()
    };
    let graph = build(plan);
    #[cfg(feature = "test-instrumentation")]
    {
        let selected = super::super::value_flow_metrics::tests::builtin_callback_sites();
        let expected = graph
            .resolved_call_sites()
            .iter()
            .filter(|site| matches!(site.caller.as_deref(), Some("owner" | "secondOwner")))
            .count();
        assert_eq!(selected, expected);
        assert!(selected < graph.resolved_call_sites().len());
        assert!(super::super::value_flow_metrics::callback_ownership_cache_initialized());
    }
    let ordinary = build(GraphBuildPlan {
        assertion_owners: false,
        ..plan
    });
    assert_eq!(ordinary.resolved_call_sites(), graph.resolved_call_sites());
    #[cfg(feature = "test-instrumentation")]
    assert!(!super::super::value_flow_metrics::callback_ownership_cache_initialized());
    let callback = graph
        .assertion_call_sites()
        .iter()
        .find(|site| site.caller.as_deref() == Some("withOwner") && site.source_callee == "handler")
        .unwrap();
    assert!(!callback.has_unknown_target(), "{callback:?}");
    let names: std::collections::BTreeSet<_> = callback
        .target_nodes()
        .filter_map(|target| match target {
            NodeId::Symbol { symbol, .. } => Some(symbol.as_ref()),
            _ => None,
        })
        .collect();
    assert_eq!(names, ["owner", "secondOwner"].into_iter().collect());
    let unresolved = graph
        .assertion_call_sites()
        .iter()
        .find(|site| {
            site.caller.as_deref() == Some("unresolved") && site.source_callee == "handler"
        })
        .unwrap();
    assert!(unresolved.has_unknown_target(), "{unresolved:?}");
}
