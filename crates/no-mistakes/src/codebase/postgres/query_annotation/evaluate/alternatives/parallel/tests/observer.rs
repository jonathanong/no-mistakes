use super::*;
use crate::diagnostics::{with_observer, InvocationObserver};
#[test]
fn eligible_arms_belong_to_the_request_observer_and_jobs1_bypasses_forks() {
    let (path, _, file) = support::fixture("no-deletion");
    let files = support::files(&path, &file.query_annotation[0].1, &file.embedded_sql[0].1);
    let serial_observer = InvocationObserver::new(true);
    with_observer(Some(serial_observer.clone()), || {
        support::evaluator(&files).module_environment(&path);
    });
    let serial_copies = serial_observer
        .snapshot()
        .work
        .get("postgres.annotation.dense_arena_slots_copied")
        .copied()
        .unwrap_or(0);
    for jobs in [1, 2, 4] {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build()
            .unwrap();
        let observer = InvocationObserver::new(true);
        pool.install(|| {
            with_observer(Some(observer.clone()), || {
                let mut evaluator = support::evaluator(&files);
                evaluator.enable_parallel_arms();
                evaluator.module_environment(&path);
            })
        });
        let work = observer.snapshot().work;
        assert_eq!(
            work.get("postgres.annotation.parallel_factory_arms")
                .copied()
                .unwrap_or(0),
            if jobs == 1 { 0 } else { 2 }
        );
        assert!(
            work.get("postgres.annotation.dense_arena_slots_copied")
                .copied()
                .unwrap_or(0)
                <= serial_copies
        );
        let before = work;
        pool.install(|| {
            with_observer(None, || {
                let mut evaluator = support::evaluator(&files);
                evaluator.enable_parallel_arms();
                evaluator.module_environment(&path);
            })
        });
        assert_eq!(
            observer.snapshot().work,
            before,
            "disabled sibling must not contaminate an observer"
        );
        assert!(crate::diagnostics::current().is_none());
    }
}
