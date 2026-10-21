use super::*;
use crate::diagnostics::with_observer;

#[test]
fn disabled_walk_creates_no_diagnostic_state() {
    with_observer(None, || assert!(Walk::new().is_none()));
    let observer = InvocationObserver::new(false);
    with_observer(Some(observer.clone()), || assert!(Walk::new().is_none()));
    assert!(observer.snapshot().work.is_empty());
    let progress_only = InvocationObserver::with_progress(false, true);
    with_observer(Some(progress_only.clone()), || {
        assert!(Walk::new().is_none())
    });
    assert!(progress_only.snapshot().work.is_empty());
}

#[test]
fn verbose_counts_revisits_without_a_uniqueness_map() {
    let observer = InvocationObserver::new(true);
    with_observer(Some(observer.clone()), || {
        let values = vec![Value::Primitive].into();
        let containers = [
            Value::Aggregate(values),
            Value::References(vec![Value::Primitive].into()),
            Value::Object(vec![Value::Primitive].into()),
            Value::Record(vec![Value::Primitive].into(), Default::default()),
            Value::Joined(vec![Value::Primitive].into()),
            Value::Possible(vec![Value::Primitive].into()),
        ];
        let mut walk = Walk::new().unwrap();
        walk.comparison();
        for container in &containers {
            walk.visit(container);
            walk.visit(&container.clone());
        }
        walk.visit(&Value::Primitive);
        assert_eq!(walk.nodes, 13);
        assert_eq!(walk.containers, 12);
        walk.complete();
    });
    let work = observer.snapshot().work;
    assert_eq!(work["postgres.annotation.prefix_nodes"], 13);
    assert_eq!(work["postgres.annotation.prefix_containers"], 12);
    assert_eq!(work["postgres.annotation.prefix_walks_completed"], 1);
}

#[test]
fn complete_and_partial_batches_account_for_every_visit() {
    for count in [0, INTERVAL, INTERVAL + 1] {
        let observer = InvocationObserver::with_progress(true, true);
        with_observer(Some(observer.clone()), || {
            let mut walk = Walk::new().unwrap();
            for _ in 0..count {
                walk.visit(&Value::Primitive);
            }
        });
        let work = observer.snapshot().work;
        if count == 0 {
            assert!(work.is_empty());
        } else {
            assert_eq!(work["postgres.annotation.prefix_nodes"], count);
            assert_eq!(
                work["postgres.annotation.prefix_observer_batches"],
                count.div_ceil(INTERVAL)
            );
        }
    }
}

#[test]
fn many_comparisons_publish_one_nonempty_batch_after_the_arm() {
    let observer = InvocationObserver::new(true);
    with_observer(Some(observer.clone()), || {
        let mut walk = Walk::new().unwrap();
        for _ in 0..128 {
            walk.comparison();
            walk.visit(&Value::Primitive);
            walk.complete();
        }
        assert!(observer.snapshot().work.is_empty());
    });
    let work = observer.snapshot().work;
    assert_eq!(work["postgres.annotation.prefix_compare"], 128);
    assert_eq!(work["postgres.annotation.prefix_walks"], 128);
    assert_eq!(work["postgres.annotation.prefix_walks_completed"], 128);
    assert_eq!(work["postgres.annotation.prefix_observer_batches"], 1);
}

#[test]
fn unwinding_publishes_started_but_incomplete_comparisons() {
    let observer = InvocationObserver::new(true);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        with_observer(Some(observer.clone()), || {
            let mut walk = Walk::new().unwrap();
            walk.comparison();
            walk.visit(&Value::Primitive);
            panic!("incomplete comparison");
        });
    }));
    assert!(result.is_err());
    let work = observer.snapshot().work;
    assert_eq!(work["postgres.annotation.prefix_walks"], 1);
    assert_eq!(work["postgres.annotation.prefix_walks_completed"], 0);
    assert_eq!(work["postgres.annotation.prefix_observer_batches"], 1);
}

#[test]
fn verbose_only_large_walk_waits_until_arm_end_to_publish() {
    let observer = InvocationObserver::new(true);
    with_observer(Some(observer.clone()), || {
        let mut walk = Walk::new().unwrap();
        for _ in 0..=INTERVAL {
            walk.visit(&Value::Primitive);
        }
        assert!(observer.snapshot().work.is_empty());
    });
    let work = observer.snapshot().work;
    assert_eq!(work["postgres.annotation.prefix_nodes"], INTERVAL + 1);
    assert_eq!(work["postgres.annotation.prefix_observer_batches"], 1);
}
