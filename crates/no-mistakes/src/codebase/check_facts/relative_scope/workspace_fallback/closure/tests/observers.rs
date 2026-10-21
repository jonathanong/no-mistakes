use super::*;

#[test]
fn legacy_none_skips_prepass_and_records_no_closure_work() {
    let fixture = Fixture::new("workspace-fallback-transitive-chain");
    let observer = InvocationObserver::new(true);
    with_observer(Some(observer.clone()), || {
        let mut fallback = fixture.fallback(None);
        fallback.prepare(&fixture.files, |_, _| {
            panic!("legacy prepass must be skipped")
        });
        assert!(fallback.prepared.is_empty());
        assert!(fallback.importers.is_none());
    });
    assert!(observer.snapshot().work.is_empty());
}

#[test]
fn verbose_work_counts_match_actual_closure_and_off_has_no_observer() {
    let fixture = Fixture::new("workspace-fallback-transitive-graph");
    let seeds = PathSet::from_iter([fixture.path("src/a.mts")]);
    with_observer(None, || {
        let mut fallback = fixture.fallback(Some(&seeds));
        assert!(fallback.observer.is_none());
        fallback.prepare(&fixture.files, |_, _| None);
        assert_eq!(fallback.prepared.len(), fixture.files.len());
    });
    let observer = InvocationObserver::new(true);
    with_observer(Some(observer.clone()), || {
        fixture
            .fallback(Some(&seeds))
            .prepare(&fixture.files, |_, _| None)
    });
    let work = observer.snapshot().work;
    for (metric, expected) in [
        ("seeds", 1),
        ("resolutions", 8),
        ("workspace_edges", 7),
        ("modules", 5),
        ("scanned_modules", 7),
        ("importers", 5),
    ] {
        assert_eq!(
            work.get(format!("postgres.annotation.closure_{metric}").as_str()),
            Some(&expected)
        );
    }
}
