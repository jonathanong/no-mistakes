#[test]
fn count_skips_zero_and_records_observer_work() {
    super::count("postgres.mutable_sql_registry.producers", 0);
    let observer = crate::diagnostics::InvocationObserver::new(true);
    crate::diagnostics::with_observer(Some(observer.clone()), || {
        super::count("postgres.mutable_sql_registry.producers", 2);
    });
    assert_eq!(
        observer
            .snapshot()
            .work
            .get("postgres.mutable_sql_registry.producers")
            .copied(),
        Some(2)
    );
}
