#[test]
fn graph_gates_full_domain_and_check_preflight_counts() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/graph-gates")
        .canonicalize()
        .expect("graph-gates performance fixture should exist");
    let files = crate::codebase::ts_source::discover_visible_paths(&root)
        .into_iter()
        .filter(|path| {
            matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("ts" | "tsx" | "mts")
            )
        })
        .collect::<Vec<_>>();
    let (plan, context) =
        crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan_with_config(
            &root,
            crate::codebase::dependencies::graph::GraphBuildPlan::all().with_symbols(true),
            Some(&root.join(".no-mistakes.yml")),
        );
    let facts =
        crate::codebase::ts_source::facts::collect_ts_facts_with_context(&files, plan, &context);
    assert!(
        facts
            .values()
            .all(|file| file.operational_error.is_none() && file.parse_error.is_none()),
        "full-domain extract must succeed for every graph-gates source"
    );
    let route_refs = facts
        .values()
        .map(|file| file.route_refs.len())
        .sum::<usize>();
    let backend_routes = facts
        .values()
        .map(|file| file.backend_routes.len())
        .sum::<usize>();
    let queue_usage = facts
        .values()
        .filter(|file| file.queue_usage.is_some())
        .count();
    let http_calls = facts
        .values()
        .map(|file| file.http_calls.len())
        .sum::<usize>();
    let process_spawns = facts
        .values()
        .map(|file| file.process_spawns.len())
        .sum::<usize>();
    let react = facts
        .values()
        .map(|file| file.react_components.len())
        .sum::<usize>();
    let symbol_references = facts
        .values()
        .map(|file| file.symbol_references.len())
        .sum::<usize>();
    let check = check_json(&root).expect("graph-gates check should succeed");
    let check_value: serde_json::Value =
        serde_json::from_str(&check).expect("graph-gates check report should be JSON");
    assert_eq!(facts.len(), 75);
    assert_eq!(route_refs, 12);
    assert_eq!(backend_routes, 9);
    assert_eq!(queue_usage, 75);
    assert_eq!(http_calls, 13);
    assert_eq!(process_spawns, 4);
    assert_eq!(react, 19);
    assert_eq!(symbol_references, 310);
    assert_eq!(check_value.as_object().map(|value| value.len()), Some(7));
}

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
