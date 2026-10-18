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
