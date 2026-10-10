use super::*;

#[test]
fn builder_index_reuses_snapshots_and_follows_every_write() {
    let prefix = |id| Value::Prefix("SELECT 1".into(), true, Some(id));
    let ids = FxHashSet::from_iter([7]);
    let mut scope = Scope::from(FxHashMap::from_iter([
        ("direct".into(), prefix(7)),
        (
            "nested".into(),
            Value::Promise(Box::new(Value::Evaluated(
                Box::new(Value::Aggregate(
                    vec![Value::Possible(
                        vec![Value::Joined(vec![prefix(8)].into())].into(),
                    )]
                    .into(),
                )),
                true,
            ))),
        ),
        ("plain".into(), Value::Unknown),
    ]));
    let snapshot = scope.clone();
    assert!(scope.contains_builders(&ids));
    let index = scope.0.builders.get().unwrap() as *const _;
    assert!(snapshot.contains_builders(&FxHashSet::from_iter([8])));
    assert_eq!(index, snapshot.0.builders.get().unwrap() as *const _);
    assert!(!scope.contains_builders(&FxHashSet::from_iter([9])));
    assert!(scope == snapshot);
    assert_eq!((&scope).into_iter().count(), 3);

    scope.insert("direct".into(), Value::Unknown);
    assert!(!scope.contains_builders(&ids));
    assert!(snapshot.contains_builders(&ids));
    assert!(scope != snapshot);
    for (_, value) in &mut scope {
        *value = Value::Unknown;
    }
    assert!(!scope.contains_builders(&FxHashSet::from_iter([8])));
    // Unique-owner writes must clear the same index as detached writes.
    scope.insert("fresh".into(), prefix(9));
    assert!(scope.contains_builders(&FxHashSet::from_iter([9])));
    assert!(!Scope::default().contains_builders(&ids));
}

#[test]
fn builder_index_handles_shared_container_dag_and_detached_mutation() {
    let leaf = Value::Possible(vec![Value::Prefix("SELECT 1".into(), true, Some(17))].into());
    let branch = Value::Aggregate(vec![leaf; 16].into());
    let mut scope = Scope::from(FxHashMap::from_iter([(
        "shared".into(),
        Value::Joined(vec![branch; 16].into()),
    )]));
    let original = scope.clone();
    let old = FxHashSet::from_iter([17]);
    assert!(scope.contains_builders(&old));
    assert!(original.contains_builders(&old));

    scope.insert(
        "new".into(),
        Value::Prefix("SELECT 2".into(), true, Some(18)),
    );
    let new = FxHashSet::from_iter([18]);
    assert!(scope.contains_builders(&new));
    assert!(!original.contains_builders(&new));
    assert!(original.contains_builders(&old));
}
