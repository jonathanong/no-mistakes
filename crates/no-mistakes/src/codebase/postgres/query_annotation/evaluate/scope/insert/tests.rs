use super::*;
use crate::diagnostics::{with_observer, InvocationObserver};

fn indexed() -> Scope {
    let mut values = (0..128)
        .map(|n| (format!("scalar{n}"), Value::Unknown))
        .collect::<FxHashMap<_, _>>();
    values.insert("argument".into(), Value::Arguments(7));
    Scope::from(values)
}

fn replacements(mut scope: Scope, optimized: bool) -> (Scope, u64) {
    let observer = InvocationObserver::new(true);
    with_observer(Some(observer.clone()), || {
        scope.references();
        scope.reference_bindings().count();
        for n in 0..16 {
            let snapshot = scope.clone();
            if optimized {
                scope.insert("scalar0".into(), Value::Number(n));
            } else {
                scope.deref_mut().insert("scalar0".into(), Value::Number(n));
            }
            scope.references();
            scope.reference_bindings().count();
            assert!(snapshot["scalar0"] != Value::Number(n));
        }
    });
    (
        scope,
        observer.snapshot().work["postgres.annotation.prefix_binding_scans"],
    )
}

#[test]
fn repeated_scalar_replacements_share_indexes_and_reduce_actual_scans() {
    let (raw, raw_scans) = replacements(indexed(), false);
    let (cached, cached_scans) = replacements(indexed(), true);
    assert!(raw == cached);
    assert_eq!(cached_scans, 129);
    assert_eq!(raw_scans, 17 * cached_scans);
    let raw_names = raw
        .iter()
        .filter(|(_, value)| value.contains_reference())
        .map(|(name, _)| name)
        .collect::<Vec<_>>();
    let cached_names = cached
        .reference_bindings()
        .map(|(name, _)| name)
        .collect::<Vec<_>>();
    assert_eq!(raw_names, cached_names);
}

#[test]
fn same_reference_membership_preserves_order_but_changed_edges_invalidate() {
    let mut scope = indexed();
    scope.reference_bindings().count();
    scope.references();
    let original = scope.clone();
    scope.insert("argument".into(), Value::Array(8));
    assert!(Arc::ptr_eq(
        scope.0.reference_binding_names.get().unwrap(),
        original.0.reference_binding_names.get().unwrap()
    ));
    assert!(scope.cached_references().is_none());
    assert_eq!(scope.references().arguments, FxHashSet::from_iter([8]));
    assert_eq!(original.references().arguments, FxHashSet::from_iter([7]));
    scope.insert("argument".into(), Value::Unknown);
    assert!(scope.0.reference_binding_names.get().is_none());
    assert!(!scope.contains_reference());
    scope.insert("argument".into(), Value::Arguments(9));
    assert!(scope.contains_reference());
    assert_eq!(scope.references().arguments, FxHashSet::from_iter([9]));
}

#[test]
fn equivalent_direct_projections_share_the_owned_edge_and_builder_indexes() {
    let callback = super::super::reference_binding_tests::saved_function();
    let values: crate::codebase::postgres::query_annotation::evaluate::value::Values =
        vec![Value::Arguments(7)].into();
    for (before, after) in [
        (Value::Arguments(7), Value::Array(7)),
        (
            Value::Prefix("old".into(), true, Some(9)),
            Value::Prefix("new".into(), false, Some(9)),
        ),
        (
            Value::Record(values.clone(), Vec::new()),
            Value::Object(values),
        ),
        (callback.clone(), Value::Promise(Box::new(callback))),
        (
            Value::Unknown,
            Value::Evaluated(Box::new(Value::Number(1)), false),
        ),
    ] {
        let mut scope = Scope::from(FxHashMap::from_iter([("value".into(), before)]));
        scope.references();
        scope.contains_builders(&FxHashSet::from_iter([9]));
        let snapshot = scope.clone();
        scope.insert("value".into(), after);
        assert!(std::ptr::eq(scope.references(), snapshot.references()));
        assert!(Arc::ptr_eq(
            scope.0.builders.get().unwrap(),
            snapshot.0.builders.get().unwrap()
        ));
    }
}

#[test]
fn new_keys_and_unrestricted_mutations_drop_ordered_indexes() {
    let mut scope = indexed();
    for n in 0..64 {
        scope.reference_bindings().count();
        scope.insert(format!("new{n}"), Value::Unknown);
        assert!(scope.0.reference_binding_names.get().is_none());
        let expected = scope
            .iter()
            .filter(|(_, value)| value.contains_reference())
            .map(|(name, _)| name)
            .collect::<Vec<_>>();
        assert_eq!(
            scope
                .reference_bindings()
                .map(|(name, _)| name)
                .collect::<Vec<_>>(),
            expected
        );
    }
    scope.references();
    *scope.get_mut("argument").unwrap() = Value::Array(11);
    assert!(scope.cached_references().is_none());
    assert!(scope.0.reference_binding_names.get().is_none());
    assert_eq!(scope.references().arguments, FxHashSet::from_iter([11]));
}
