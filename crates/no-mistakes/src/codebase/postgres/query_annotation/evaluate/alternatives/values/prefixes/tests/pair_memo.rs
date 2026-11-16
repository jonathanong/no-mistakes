use super::{fixture, progress, Value};
use crate::codebase::postgres::query_annotation::evaluate::alternatives::values::{
    changes, ComparisonMemo,
};
use crate::diagnostics::{with_observer, InvocationObserver};
use crate::fx::FxHashSet;

fn compare(
    before: &fixture::Case,
    after: &fixture::Case,
    optimized: bool,
    definite: &FxHashSet<(u64, usize)>,
) -> (FxHashSet<u64>, u64, usize, usize) {
    let aliases = |value: &Value| {
        [
            value.clone(),
            value.clone(),
            Value::Promise(Box::new(value.clone())),
            Value::Evaluated(Box::new(value.clone()), false),
        ]
    };
    let previous = aliases(&before.root);
    let current = aliases(&after.root);
    let observer = InvocationObserver::new(true);
    let mut changed = FxHashSet::default();
    let mut hits = 0;
    let mut misses = 0;
    with_observer(Some(observer.clone()), || {
        let mut memo = ComparisonMemo::default();
        let mut work = progress::Walk::new();
        for (before_value, after_value) in previous.iter().zip(&current) {
            if optimized {
                match memo.repeated(before_value, after_value) {
                    Some(true) => {
                        hits += 1;
                        continue;
                    }
                    Some(false) => misses += 1,
                    None => {}
                }
            }
            changes(
                before_value,
                after_value,
                before.arena(),
                after.arena(),
                &mut changed,
                definite,
                &mut work,
            );
        }
    });
    let walks = observer.snapshot().work["postgres.annotation.prefix_walks"];
    (changed, walks, hits, misses)
}

#[test]
fn repeated_positive_and_negative_pairs_reuse_the_actual_comparison() {
    let before = fixture::load("arena_before");
    let after = fixture::load("arena_after");
    for current in [&before, &after] {
        let raw = compare(&before, current, false, &current.definite);
        let cached = compare(&before, current, true, &current.definite);
        assert_eq!(raw.0, cached.0);
        assert_eq!((raw.1, cached.1, cached.2, cached.3), (4, 1, 3, 1));
    }
    assert_eq!(
        compare(&before, &after, true, &after.definite).0,
        FxHashSet::from_iter([1])
    );
    assert!(compare(&before, &before, true, &before.definite)
        .0
        .is_empty());
}

#[test]
fn new_phases_recompute_changed_arenas_cycles_sparse_slots_and_deletion_masks() {
    for name in ["arena_before", "reentrant_record", "sparse_deleted_slots"] {
        let case = fixture::load(name);
        let raw = compare(&case, &case, false, &case.definite);
        let cached = compare(&case, &case, true, &case.definite);
        assert_eq!(cached.0, raw.0);
        assert_eq!((cached.1, cached.2, cached.3), (1, 3, 1));
    }
    let before = fixture::load("arena_before");
    let after = fixture::load("arena_after");
    let deleted = FxHashSet::from_iter([(1, 0)]);
    assert!(compare(&before, &after, true, &deleted).0.is_empty());
    assert_eq!(
        compare(&before, &after, true, &after.definite).0,
        FxHashSet::from_iter([1])
    );
}

#[test]
fn argument_and_array_roots_share_keys_but_direct_prefixes_do_not() {
    let mut memo = ComparisonMemo::default();
    assert_eq!(
        memo.repeated(&Value::Arguments(7), &Value::Array(8)),
        Some(false)
    );
    assert_eq!(
        memo.repeated(&Value::Array(7), &Value::Arguments(8)),
        Some(true)
    );
    assert_eq!(
        memo.repeated(&Value::Arguments(8), &Value::Array(7)),
        Some(false)
    );
    let scalar = Value::Prefix("changed".into(), true, Some(1));
    assert_eq!(memo.repeated(&scalar, &scalar), None);
    assert_eq!(memo.repeated(&Value::Arguments(7), &scalar), None);
    assert_eq!(memo.repeated(&Value::Unknown, &Value::Arguments(7)), None);
}
