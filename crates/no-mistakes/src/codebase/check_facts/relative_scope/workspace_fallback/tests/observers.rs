use super::*;

fn exercise(fixture: &Fixture) {
    let from = fixture.path("caller.mts");
    let selected = PathSet::from_iter([from.clone()]);
    let fallback = fixture.fallback(Some(&selected));
    assert!(fallback
        .resolve("@fixture/db", &from, |_, _| None)
        .is_some());
    assert_eq!(
        fallback.resolve("@fixture/db", &fixture.path("other.mts"), |_, _| None),
        None
    );
    // Neither primary hits nor unresolved imports are fallback edges.
    assert!(fallback
        .resolve("@fixture/db", &from, |_, _| Some(from.clone()))
        .is_some());
    assert_eq!(
        fallback.resolve("@fixture/missing", &from, |_, _| None),
        None
    );
}

#[test]
fn disabled_and_timings_only_have_no_fallback_counters() {
    let fixture = Fixture::new(&[]);
    crate::diagnostics::with_observer(None, || {
        assert!(fixture.fallback(None).observer.is_none());
        exercise(&fixture);
    });
    let timings = InvocationObserver::new(false);
    crate::diagnostics::with_observer(Some(timings.clone()), || {
        assert!(fixture.fallback(None).observer.is_none());
        exercise(&fixture);
    });
    assert!(timings.snapshot().work.is_empty());
}

#[test]
fn verbose_counts_only_actual_fallback_acceptance_and_rejection() {
    let fixture = Fixture::new(&[]);
    let verbose = InvocationObserver::new(true);
    crate::diagnostics::with_observer(Some(verbose.clone()), || exercise(&fixture));
    let work = verbose.snapshot().work;
    assert_eq!(
        work.get("postgres.annotation.workspace_fallback_edges"),
        Some(&1)
    );
    assert_eq!(
        work.get("postgres.annotation.workspace_fallback_blocked"),
        Some(&1)
    );
    assert_eq!(work.len(), 2);
}
