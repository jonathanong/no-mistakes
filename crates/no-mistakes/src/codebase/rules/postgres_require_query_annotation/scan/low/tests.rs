use crate::codebase::check_facts::{collect_check_facts, CheckFactPlan};
use crate::codebase::rules::postgres_require_query_annotation::{compile_options, Options};

#[test]
fn borrowed_observer_counts_calls_on_a_worker_without_diagnostic_tls() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("low-heads");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let files = vec![root.join("direct-only.mts")];
    let opts = compile_options(&Options {
        import_specifier: "@bench/db".into(),
        executor_names: vec!["read".into()],
        ..Default::default()
    })
    .unwrap();
    let facts = collect_check_facts(
        root,
        files.clone(),
        CheckFactPlan {
            query_annotation_heads: true,
            embedded_sql_options: vec![opts.embedded.clone()],
            ..Default::default()
        },
    );
    let expected_calls = facts.ts.get(&files[0]).unwrap().embedded_sql[0]
        .1
        .calls
        .len();
    assert_eq!(expected_calls, 2);
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let (without, with) = std::thread::scope(|scope| {
        scope
            .spawn(|| {
                // A stolen worker has no caller TLS. Both scans must still
                // agree, and the explicitly borrowed observer must see work.
                assert!(crate::diagnostics::current().is_none());
                (
                    super::scan(root, &opts, &files, &facts, None).unwrap(),
                    super::scan(root, &opts, &files, &facts, Some(&observer)).unwrap(),
                )
            })
            .join()
            .unwrap()
    });
    assert_eq!(without.finish(), with.finish());
    assert_eq!(
        observer
            .snapshot()
            .work
            .get("postgres.annotation.low.calls"),
        Some(&(expected_calls as u64))
    );
}
