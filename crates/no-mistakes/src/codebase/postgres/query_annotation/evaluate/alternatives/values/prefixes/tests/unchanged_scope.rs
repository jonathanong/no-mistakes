use super::{fixture, FxHashMap, FxHashSet, Value};
use crate::codebase::postgres::query_annotation::evaluate::Scope;
use crate::diagnostics::{with_observer, InvocationObserver};

fn scope(value: Value) -> Scope {
    Scope::from(FxHashMap::from_iter([("binding".into(), value)]))
}

fn unchanged(before: &Scope, after: &Scope) -> bool {
    super::super::super::unchanged_without_arguments(before, after)
}

fn compare(
    before: &Scope,
    after: &Scope,
    previous: &fixture::Case,
    current: &fixture::Case,
    optimized: bool,
) -> (FxHashSet<u64>, u64) {
    let observer = InvocationObserver::new(true);
    let mut changed = FxHashSet::default();
    with_observer(Some(observer.clone()), || {
        if optimized && unchanged(before, after) {
            return;
        }
        let mut work = super::super::progress::Walk::new();
        for (name, value) in before.reference_bindings() {
            super::super::super::changes(
                value,
                super::super::super::super::bindings::binding_or_unproven(after, name),
                previous.arena(),
                current.arena(),
                &mut changed,
                &current.definite,
                &mut work,
            );
        }
    });
    let walks = observer
        .snapshot()
        .work
        .get("postgres.annotation.prefix_walks")
        .copied()
        .unwrap_or(0);
    (changed, walks)
}

#[test]
fn warm_unchanged_immutable_scope_skips_actual_prefix_walks() {
    for name in ["all_container_kinds", "shared_prefix_depth30"] {
        let case = fixture::load(name);
        let before = scope(case.root.clone());
        let after = before.clone();
        assert!(before.contains_reference());
        assert!(before.cached_references().is_none());
        assert!(!unchanged(&before, &after));
        let cold = compare(&before, &after, &case, &case, true);
        assert!(cold.1 > 0);
        assert!(before.cached_references().is_none());
        assert!(before.references().arguments.is_empty());
        let legacy = compare(&before, &after, &case, &case, false);
        let optimized = compare(&before, &after, &case, &case, true);
        assert_eq!(optimized.0, legacy.0);
        assert!(optimized.0.is_empty());
        assert!(legacy.1 > 0);
        assert_eq!(optimized.1, 0);
        assert!(before.same_bindings(&after));
        assert!(*before == *after);
    }
}

#[test]
fn changed_binding_detaches_and_compares_exact_prefix_mutation() {
    let case = fixture::load("all_container_kinds");
    let before = scope(case.root.clone());
    before.references();
    let mut after = before.clone();
    after.insert(
        "binding".into(),
        Value::Prefix("changed".into(), true, Some(1)),
    );
    assert!(!unchanged(&before, &after));
    let actual = compare(&before, &after, &case, &case, true);
    assert_eq!(actual.0, FxHashSet::from_iter([1]));
    assert_eq!(actual, compare(&before, &after, &case, &case, false));
    assert!(after.cached_references().is_none());
}

#[test]
fn warm_argument_array_cycle_and_sparse_scopes_keep_original_walks() {
    for name in [
        "arena_before",
        "shared_arguments_depth30",
        "reentrant_record",
        "sparse_deleted_slots",
    ] {
        let case = fixture::load(name);
        let id = *case.objects.keys().next().unwrap();
        for value in [case.root.clone(), Value::Array(id)] {
            let before = scope(value);
            let after = before.clone();
            assert!(!before.references().arguments.is_empty());
            assert!(!unchanged(&before, &after));
            let actual = compare(&before, &after, &case, &case, true);
            assert_eq!(actual, compare(&before, &after, &case, &case, false));
            assert!(actual.1 > 0);
            assert!(actual.0.is_empty());
        }
    }
    let previous = fixture::load("arena_before");
    let current = fixture::load("arena_after");
    let before = scope(previous.root.clone());
    let after = before.clone();
    before.references();
    let actual = compare(&before, &after, &previous, &current, true);
    assert_eq!(actual.0, FxHashSet::from_iter([1]));
    assert_eq!(actual, compare(&before, &after, &previous, &current, false));
}
