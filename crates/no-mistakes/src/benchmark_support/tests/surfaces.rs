use super::*;

#[test]
fn react_traits_many_components_fixture_locks_trait_counts() {
    let fixture = react_traits_many_components_fixture();
    assert_eq!(
        analyze_react_traits_file(&fixture),
        ReactTraitsSummary {
            components: 32,
            with_state: 4,
            with_props: 8,
            with_memo: 4,
            with_context: 4,
            with_suspense: 4,
            with_fetch: 4,
            with_children: 4,
        }
    );
}

#[test]
fn scoped_resolver_fixture_caches_one_selection_per_importer() {
    let fixture = scoped_resolver_selection_fixture();

    assert_eq!(
        resolve_repeated_scoped_imports(&fixture, 64),
        ScopedResolverSelectionSummary {
            resolved: 64,
            selection_builds: 1,
        }
    );
}

#[test]
fn scoped_resolver_construction_benchmark_preserves_its_result_contract() {
    let fixture = scoped_resolver_selection_fixture();
    assert_eq!(build_repeated_scoped_resolvers(&fixture, 4), 4);
}

#[test]
fn bench_shard_unset_and_general_memory_run_every_named_surface() {
    let named = [
        CHECK,
        OBSERVER,
        TESTS_PLAN,
        GRAPH_CORE,
        GRAPH_GATES,
        LANGUAGE_FRONTENDS,
        NATIVE_FRONTENDS,
        GRAPH_FINALIZATION,
        GRAPH_PRODUCTION,
        QUERY,
    ];
    for current in [None, Some(""), Some(GENERAL_MEMORY)] {
        assert_eq!(parse_bench_shard(current), Ok(BenchShard::All));
        for requested in named {
            assert_eq!(shard_should_run(requested, current), Ok(true));
        }
    }
}

#[test]
fn bench_shard_named_values_isolate_one_surface() {
    let named = [
        CHECK,
        OBSERVER,
        TESTS_PLAN,
        GRAPH_CORE,
        GRAPH_GATES,
        LANGUAGE_FRONTENDS,
        NATIVE_FRONTENDS,
        GRAPH_FINALIZATION,
        GRAPH_PRODUCTION,
        QUERY,
    ];
    for requested in named {
        assert_eq!(
            parse_bench_shard(Some(requested)),
            Ok(BenchShard::Named(requested))
        );
        assert_eq!(shard_should_run(requested, Some(requested)), Ok(true));
        for other in named {
            if other != requested {
                assert_eq!(shard_should_run(other, Some(requested)), Ok(false));
            }
        }
    }
}

#[test]
fn bench_shard_rejects_unknown_values() {
    for current in [Some("grap"), Some("GRAPH"), Some("cpu")] {
        let err = parse_bench_shard(current).expect_err("unknown shard names must fail");
        assert!(err.contains("unknown NO_MISTAKES_BENCH_SHARD"));
        assert!(shard_should_run(GRAPH_CORE, current).is_err());
    }
}

#[test]
fn callable_file_index_fixture_constructs_and_probes_call_sites() {
    let fixture = callable_file_index_fixture(16);
    let constructed = crate::codebase::dependencies::graph::benchmark_construct_callable_file_index(
        &fixture.facts,
    );
    assert!(
        constructed >= 16,
        "unclamped construction must produce at least the requested entries"
    );
    let summary = construct_callable_file_index(&fixture);
    assert_eq!(summary.entries, constructed.max(fixture.entries));
    assert_eq!(probe_call_site_files(0), 0);
    assert!(probe_call_site_files(4) > 0);
}

#[test]
fn scoped_callable_fixtures_preserve_dense_and_sparse_probe_semantics() {
    for sparse in [false, true] {
        let fixture = scoped_callable_fixture(16, sparse);
        // Count bindings, not distinct scopes: 16 aliases, 17 ordinary/class
        // bindings, 16 deep-frame bindings, and one class binding.
        assert_eq!(construct_scoped_callable_index(&fixture), 50);
        let index = prepare_scoped_callable_index(&fixture);
        assert_eq!(index.local_id(fixture.scope, &fixture.binding), Some(0));
        assert_eq!(
            index.local_id(fixture.deep_scope, &fixture.binding),
            Some(0)
        );
        assert!(index.local_id(fixture.scope, "missing").is_none());
        assert!(index.local_id(fixture.deep_scope, "missing").is_none());
        assert!(index.alias_resolves(fixture.scope, &fixture.alias));
        assert!(!index.alias_resolves(fixture.scope, "missing"));
        assert!(index.binding_live(fixture.scope, &fixture.binding));
        assert_eq!(
            index.class_id(fixture.class_scope, "Class0.m0"),
            Some(2_000_000)
        );
        assert!(index
            .class_id(fixture.class_scope, "Class0.missing")
            .is_none());
        assert!(index.class_id(fixture.class_scope, "missing").is_none());
    }
}
