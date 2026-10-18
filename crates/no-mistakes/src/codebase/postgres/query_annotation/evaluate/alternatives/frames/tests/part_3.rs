#[test]
fn compaction_releases_dead_slots_without_remapping_live_handles() {
    let mut scopes: ScopeArena = vec![
        Scope::from(FxHashMap::from_iter([(
            "arguments".into(),
            Value::Arguments(42),
        )])),
        Scope::default(),
        Scope::from(FxHashMap::from_iter([
            ("callback".into(), function_value(2)),
            (
                "builder".into(),
                Value::Prefix("SELECT 1".into(), true, Some(7)),
            ),
        ])),
    ]
    .into();
    assert!(scopes[2].contains_builders(&FxHashSet::from_iter([7])));
    let snapshot = scopes[2].clone();
    let returned_values = Values::from(vec![Value::Promise(Box::new(Value::Evaluated(
        Box::new(function_value(2)),
        true,
    )))]);
    let returned_snapshot = returned_values.clone();
    let returned = vec![Value::Possible(returned_values)];
    let mut objects = FxHashMap::from_iter([(42, vec![function_value(2), Value::Arguments(42)])]);
    let mut extras = FxHashMap::from_iter([(
        42,
        std::collections::BTreeMap::from_iter([(1000, function_value(2))]),
    )]);
    let mut mapped = FxHashMap::from_iter([(2, vec![(42, vec!["parameter".into()])])]);
    let mut fresh = FxHashMap::from_iter([(2, FxHashSet::from_iter(["parameter".into()]))]);
    let mut captured = FxHashMap::from_iter([(2, FxHashMap::from_iter([("owner".into(), 2)]))]);
    let compacted = compact_from_roots(
        &mut scopes,
        reachable::RootSeeds {
            environments: &[0],
            arguments: &[],
            transient: &[],
        },
        &returned,
        &mut mapped,
        ArenaMut {
            objects: &mut objects,
            extras: &mut extras,
        },
        &mut fresh,
        &mut captured,
    );

    assert_eq!(compacted.indices, FxHashMap::from_iter([(0, 0), (2, 2)]));
    assert!(!compacted.captured_changed);
    assert_eq!(compacted.arguments, FxHashSet::from_iter([42]));
    assert_eq!(compacted.identities, FxHashSet::from_iter([7, 42]));
    assert!(scopes.get(1).is_none());
    assert!(scopes[2].shares_storage_with(&snapshot));
    assert!(scopes[2].builder_index_initialized());
    assert_eq!(function_environment(&scopes[2]["callback"]), 2);
    let Value::Possible(values) = &returned[0] else {
        panic!("returned container disappeared")
    };
    assert_eq!(values.identity(), returned_snapshot.identity());
    assert_eq!(function_environment(&values[0]), 2);
    assert_eq!(function_environment(&objects[&42][0]), 2);
    assert_eq!(function_environment(&extras[&42][&1000]), 2);
    assert!(objects[&42][1] == Value::Arguments(42));
    assert_eq!(mapped[&2], vec![(42, vec!["parameter".to_string()])]);
    assert!(fresh[&2].contains("parameter"));
    assert_eq!(captured[&2]["owner"], 2);
}

#[test]
fn argument_roots_preserve_cycles_and_callbacks_without_copying_returns() {
    let mut scopes: ScopeArena = vec![Scope::default(), Scope::default(), Scope::default()].into();
    let returned = vec![Value::Prefix("SELECT 1".into(), true, None)];
    let Value::Prefix(text, _, _) = &returned[0] else {
        unreachable!()
    };
    let return_buffer = text.as_ptr();
    let mut objects = FxHashMap::from_iter([(42, vec![Value::Arguments(42), function_value(2)])]);
    let mut extras = FxHashMap::default();
    let mut mapped = FxHashMap::default();
    let mut fresh = FxHashMap::default();
    let mut captured = FxHashMap::default();

    let compacted = compact_from_roots(
        &mut scopes,
        reachable::RootSeeds {
            environments: &[0],
            arguments: &[42],
            transient: &[],
        },
        &returned,
        &mut mapped,
        ArenaMut {
            objects: &mut objects,
            extras: &mut extras,
        },
        &mut fresh,
        &mut captured,
    );

    assert_eq!(compacted.indices, FxHashMap::from_iter([(0, 0), (2, 2)]));
    assert_eq!(compacted.arguments, FxHashSet::from_iter([42]));
    assert_eq!(compacted.identities, FxHashSet::from_iter([42]));
    assert!(scopes.get(1).is_none());
    assert_eq!(function_environment(&objects[&42][1]), 2);
    assert!(objects[&42][0] == Value::Arguments(42));
    let Value::Prefix(text, _, _) = &returned[0] else {
        panic!("returned value changed shape")
    };
    assert_eq!(text.as_ptr(), return_buffer, "the return was not copied");
}
