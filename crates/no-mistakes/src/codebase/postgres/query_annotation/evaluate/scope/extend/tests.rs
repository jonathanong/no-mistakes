use super::*;

fn warm() -> Scope {
    let scope = Scope::from(FxHashMap::from_iter([("plain".into(), Value::Unknown)]));
    scope.references();
    scope.contains_reference();
    scope.reference_bindings().count();
    scope.contains_builders(&FxHashSet::from_iter([11]));
    scope
}

fn parity(scope: &Scope) {
    let raw = Edges::collect(scope.values());
    assert_eq!(scope.references().arguments, raw.arguments);
    assert_eq!(scope.references().environments, raw.environments);
    assert_eq!(scope.references().identities, raw.identities);
}

#[test]
fn new_direct_bindings_augment_warm_edges_without_rescanning_the_capture() {
    let mut scope = warm();
    let initial = scope.clone();
    let callback = super::super::reference_binding_tests::saved_function();
    for (name, value) in [
        ("argument", Value::Arguments(7)),
        ("callback", callback),
        ("builder", Value::Prefix("SELECT".into(), true, Some(11))),
        ("empty", Value::Number(1)),
    ] {
        scope.insert(name.into(), value);
        assert!(scope.cached_references().is_some());
        assert!(scope.0.reference_binding_names.get().is_none());
        parity(&scope);
    }
    assert!(scope.contains_reference());
    assert!(scope.contains_builders(&FxHashSet::from_iter([11])));
    assert!(!initial.contains_reference());
    assert!(initial.references().arguments.is_empty());
    assert!(initial.references().environments.is_empty());
    assert!(initial.references().identities.is_empty());
    assert!(!initial.contains_builders(&FxHashSet::from_iter([11])));
}

#[test]
fn new_container_bindings_fall_back_to_the_full_direct_edge_summary() {
    let mut scope = warm();
    scope.insert(
        "container".into(),
        Value::Record(vec![Value::Arguments(8)].into(), Vec::new()),
    );
    assert!(scope.cached_references().is_none());
    assert!(scope.0.builders.get().is_none());
    assert!(scope.contains_reference());
    parity(&scope);
}

#[test]
fn absent_shadow_removal_keeps_shared_state_and_present_removal_invalidates() {
    let mut scope = warm();
    let original = scope.clone();
    assert!(scope.remove("absent").is_none());
    assert!(scope.same_bindings(&original));
    assert!(std::ptr::eq(scope.references(), original.references()));
    assert!(scope.remove("plain").is_some());
    assert!(!scope.same_bindings(&original));
    assert!(scope.cached_references().is_none());
    assert!(scope.0.reference_binding_names.get().is_none());
    assert!(scope.is_empty());
    assert_eq!(original.len(), 1);
}
