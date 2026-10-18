use super::Counters;

#[test]
fn published_counts_preserve_zero_one_many_and_operation_identities() {
    for tasks in [0, 1, 2, 5] {
        let observer = crate::diagnostics::InvocationObserver::new(true);
        crate::diagnostics::with_observer(Some(observer.clone()), || {
            let mut counters = Counters::new();
            counters.plan(tasks.min(2));
            if tasks > 1 {
                counters.capture();
            }
            for index in 0..tasks {
                counters.task();
                counters.fresh_rebuild();
                if index != 0 {
                    counters.restore();
                }
            }
        });
        let work = observer.snapshot().work;
        let value = |name| work.get(name).copied().unwrap_or(0);
        assert_eq!(
            value("postgres.annotation.run.entrypoints.zero"),
            u64::from(tasks == 0)
        );
        assert_eq!(
            value("postgres.annotation.run.entrypoints.one"),
            u64::from(tasks == 1)
        );
        assert_eq!(
            value("postgres.annotation.run.entrypoints.many"),
            u64::from(tasks > 1)
        );
        assert_eq!(value("postgres.annotation.run.entrypoints"), tasks as u64);
        assert_eq!(
            value("postgres.annotation.run.checkpoint_captures"),
            u64::from(tasks > 1)
        );
        assert_eq!(
            value("postgres.annotation.run.checkpoint_restores"),
            tasks.saturating_sub(1) as u64
        );
        assert_eq!(
            value("postgres.annotation.run.fresh_index_rebuilds"),
            tasks as u64
        );
    }
}

#[test]
fn absent_and_disabled_observer_publish_no_maps() {
    let exercise = || {
        let mut counts = Counters::new();
        counts.plan(2);
        counts.capture();
        counts.task();
        counts.restore();
        counts.fresh_rebuild();
    };
    crate::diagnostics::with_observer(None, exercise);
    let observer = crate::diagnostics::InvocationObserver::new(false);
    crate::diagnostics::with_observer(Some(observer.clone()), exercise);
    assert!(observer.snapshot().work.is_empty());
}
