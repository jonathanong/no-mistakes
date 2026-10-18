#[test]
fn mutable_argument_cycles_extra_slots_and_deletion_are_not_cached() {
    let spec = fixture();
    let files = FxHashMap::default();
    let mut e = evaluator(&files, FxHashMap::default());
    let id = ids(&spec, "live_arguments");
    let builder = |name| Value::Prefix("SELECT 1".into(), false, Some(ids(&spec, name)));
    e.argument_objects.insert(
        id,
        vec![
            Value::Array(id),
            builder("dead_builder"),
            builder("builder"),
        ],
    );
    e.argument_extra_slots.insert(
        id,
        std::collections::BTreeMap::from_iter([(9, builder("extra_builder"))]),
    );
    e.initialize_reference_index_for_tests();
    e.definite_deleted_argument_slots.insert((id, 1));
    e.deleted_argument_slots.insert((id, Some(9)));
    let roots = vec![Value::Arguments(id), Value::Array(id)];
    let (old, _) = collect_old(&e, &roots);
    let (new, _) = collect_new(&e, &roots);
    assert_eq!(old, new);
    assert_eq!(
        new,
        FxHashSet::from_iter([id, ids(&spec, "builder"), ids(&spec, "extra_builder")])
    );
    e.write_dense_slot(id, 1, Value::Prefix("SELECT 3".into(), false, Some(11)));
    e.definite_deleted_argument_slots.remove(&(id, 1));
    let old = collect_old(&e, &roots).0;
    let new = collect_new(&e, &roots).0;
    assert_eq!(old, new);
    assert!(new.contains(&11));
    assert!(!new.contains(&ids(&spec, "dead_builder")));
}
