use super::*;

#[test]
fn direct_reference_cache_detaches_and_invalidates_for_all_mutation_paths() {
    let mut scope = Scope::from(FxHashMap::from_iter([
        ("arg".into(), Value::Arguments(7)),
        ("array".into(), Value::Array(8)),
        (
            "sql".into(),
            Value::Prefix("SELECT 1".into(), true, Some(9)),
        ),
    ]));
    let snapshot = scope.clone();
    let edges = scope.references();
    assert_eq!(edges.arguments, FxHashSet::from_iter([7, 8]));
    assert_eq!(edges.identities, FxHashSet::from_iter([9]));
    assert!(std::ptr::eq(edges, snapshot.references()));
    scope.clear();
    assert!(scope.references().arguments.is_empty());
    assert!(scope.references().identities.is_empty());
    assert_eq!(snapshot.references().arguments.len(), 2);
    scope.insert("new".into(), Value::Arguments(10));
    assert_eq!(scope.references().arguments, FxHashSet::from_iter([10]));
    for (_, value) in &mut scope {
        *value = Value::Unknown;
    }
    assert!(scope.references().arguments.is_empty());
    assert!(!scope.same_bindings(&snapshot));
    assert!(scope == Scope::from(FxHashMap::from_iter([("new".into(), Value::Unknown)])));
}
