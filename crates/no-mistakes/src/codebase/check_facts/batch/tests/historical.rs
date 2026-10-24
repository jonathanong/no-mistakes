use super::*;

pub(super) fn assert_parity(requests: &[BatchCheckFactRequest]) {
    let mut old_clones = 0;
    let old = super::super::test_support::demands_by_path::<true>(requests, &mut old_clones);
    let mut slots = demands::graph_plans(requests);
    let mut new_clones = 0;
    let new = demands::demands_by_path::<true>(requests, &mut slots, &mut new_clones);
    assert!(new_clones <= old_clones);
    assert_eq!(
        new.keys().collect::<Vec<_>>(),
        old.keys().collect::<Vec<_>>()
    );
    for (path, expected) in old {
        let actual = &new[&path];
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().zip(expected) {
            assert_eq!(actual.request, expected.request);
            let actual = actual.plan.as_ref();
            let expected = expected.plan;
            assert_eq!(actual.source, expected.source);
            assert_eq!(actual.symbols, expected.symbols);
            assert_eq!(actual.graph, expected.graph);
            assert_eq!(actual.legacy_symbol_paths, expected.legacy_symbol_paths);
            assert_eq!(actual.call_site_paths, expected.call_site_paths);
            assert_eq!(
                actual.production_import_paths,
                expected.production_import_paths
            );
            assert_eq!(actual.storybook, expected.storybook);
            assert_eq!(actual.integration, expected.integration);
            match (
                &actual.forbidden_call_arguments,
                &expected.forbidden_call_arguments,
            ) {
                (Some(actual), Some(expected)) => assert!(Arc::ptr_eq(actual, expected)),
                (None, None) => {}
                _ => panic!("argument demand changed for {}", path.display()),
            }
        }
    }
}

#[test]
fn feature_adapter_keeps_cold_demand_counts_and_disables_timed_accounting() {
    let root = root();
    let files = ["web/primary.ts", "web/helper.ts", "web/graph.ts"]
        .iter()
        .map(|p| root.join(p))
        .collect::<Vec<_>>();
    let fixture = crate::benchmark_support::prepare_fact_demands(
        &root,
        files.clone(),
        files,
        CheckFactPlan::default(),
    );
    for historical in [false, true] {
        for counted in [false, true] {
            let result =
                crate::benchmark_support::construct_fact_demands(&fixture, historical, counted);
            assert_eq!((result.0, result.1), (3, 3));
            assert_eq!(result.2, if historical && counted { 3 } else { 0 });
        }
    }
}
