use super::*;

#[test]
fn changed_compaction_remaps_shared_scopes_results_and_heap_metadata() {
    let mut scopes = vec![
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
    ];
    assert!(scopes[2].contains_builders(&FxHashSet::from_iter([7])));
    let snapshot = scopes[2].clone();
    let returned_values = Values::from(vec![Value::Promise(Box::new(Value::Evaluated(
        Box::new(function_value(2)),
        true,
    )))]);
    let returned_snapshot = returned_values.clone();
    let mut returned = vec![Value::Possible(returned_values)];
    let mut objects =
        ArgumentObjects::from_iter([(42, vec![function_value(2), Value::Arguments(42)])]);
    let mut extras = FxHashMap::from_iter([(
        42,
        std::collections::BTreeMap::from_iter([(1000, function_value(2))]),
    )]);
    let mut mapped = FxHashMap::from_iter([(2, vec![(42, vec!["parameter".into()])])]);
    let mut fresh = FxHashMap::from_iter([(2, FxHashSet::from_iter(["parameter".into()]))]);
    let mut captured =
        FxHashMap::from_iter([(2, FxHashMap::from_iter([("owner".into(), 2)]).into())]);
    let indices = compact_from_roots(
        &mut scopes,
        Roots {
            environments: &[0],
            arguments: &[],
            preserved: 0,
            transient: &[],
        },
        &mut returned,
        &mut mapped,
        ArenaMut {
            objects: &mut objects,
            extras: &mut extras,
        },
        &mut fresh,
        &mut captured,
    );
    assert_eq!(indices.indices, FxHashMap::from_iter([(0, 0), (2, 1)]));
    assert!(indices.captured_changed);
    assert_eq!(indices.arguments, FxHashSet::from_iter([42]));
    assert_eq!(indices.identities, FxHashSet::from_iter([7, 42]));
    assert_eq!(scopes.len(), 2);
    assert!(!std::ptr::eq(&*scopes[1], &*snapshot));
    assert!(!scopes[1].builder_index_initialized());
    assert!(snapshot.builder_index_initialized());
    assert_eq!(function_environment(&scopes[1]["callback"]), 1);
    assert_eq!(function_environment(&snapshot["callback"]), 2);
    let Value::Possible(values) = &returned[0] else {
        panic!("returned container disappeared")
    };
    assert_ne!(values.identity(), returned_snapshot.identity());
    assert_eq!(function_environment(&values[0]), 1);
    assert_eq!(function_environment(&returned_snapshot[0]), 2);
    assert_eq!(function_environment(&objects[&42][0]), 1);
    assert_eq!(function_environment(&extras[&42][&1000]), 1);
    assert!(objects[&42][1] == Value::Arguments(42));
    assert_eq!(mapped[&1], vec![(42, vec!["parameter".to_string()])]);
    assert!(fresh[&1].contains("parameter"));
    assert_eq!(captured[&1]["owner"], 1);
}

#[test]
fn initial_argument_roots_preserve_cycles_and_callbacks_without_copying_returns() {
    let mut scopes = vec![Scope::default(), Scope::default(), Scope::default()];
    let mut returned = vec![Value::Prefix("SELECT 1".into(), true, None)];
    let Value::Prefix(text, _, _) = &returned[0] else {
        unreachable!()
    };
    let return_buffer = text.as_ptr();
    let mut objects =
        ArgumentObjects::from_iter([(42, vec![Value::Arguments(42), function_value(2)])]);
    let mut extras = FxHashMap::default();
    let mut mapped = FxHashMap::default();
    let mut fresh = FxHashMap::default();
    let mut captured = FxHashMap::default();

    let compacted = compact_from_roots(
        &mut scopes,
        Roots {
            environments: &[0],
            arguments: &[42],
            preserved: 1,
            transient: &[],
        },
        &mut returned,
        &mut mapped,
        ArenaMut {
            objects: &mut objects,
            extras: &mut extras,
        },
        &mut fresh,
        &mut captured,
    );

    assert_eq!(compacted.indices.get(&2), Some(&1));
    assert_eq!(compacted.arguments, FxHashSet::from_iter([42]));
    assert_eq!(compacted.identities, FxHashSet::from_iter([42]));
    assert!(objects[&42][0] == Value::Arguments(42));
    assert_eq!(function_environment(&objects[&42][1]), 1);
    let Value::Prefix(text, _, _) = &returned[0] else {
        panic!("returned value changed shape")
    };
    assert_eq!(text.as_ptr(), return_buffer, "the return was not copied");
}
