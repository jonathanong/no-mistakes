use super::*;

#[test]
fn mixed_immutable_roots_keep_dense_sparse_deleted_and_cyclic_argument_state_live() {
    let spec = spec();
    let argument = id(&spec, "argument");
    let array = id(&spec, "array");
    let roots = wrappers(Value::Aggregate(
        vec![
            free_tree(&spec),
            builder(&spec, "builder"),
            Value::Arguments(argument),
            Value::Array(array),
        ]
        .into(),
    ));
    let files = FxHashMap::default();
    let mut old = evaluator(&files, FxHashMap::default());
    let mut new = evaluator(&files, FxHashMap::default());
    for e in [&mut old, &mut new] {
        e.argument_objects.insert(
            argument,
            vec![
                Value::Array(array),
                builder(&spec, "deleted"),
                free_tree(&spec),
            ],
        );
        e.argument_objects
            .insert(array, vec![Value::Arguments(argument)]);
        e.argument_extra_slots.insert(
            argument,
            std::collections::BTreeMap::from([(9, builder(&spec, "sparse"))]),
        );
        e.definite_deleted_argument_slots.insert((argument, 1));
    }
    for changed in [false, true] {
        if changed {
            for e in [&mut old, &mut new] {
                e.argument_objects.get_mut(&argument).unwrap()[2] = builder(&spec, "replacement");
                e.definite_deleted_argument_slots.remove(&(argument, 1));
                e.definite_deleted_argument_slots.insert((argument, 9));
            }
        }
        let (old_ids, _) = collect_old(&old, &roots);
        let mut visits = 0;
        let new_ids = new.collect_builder_ids(&roots, |()| visits += 1);
        let mut expected = FxHashSet::from_iter([id(&spec, "builder"), argument, array]);
        expected.insert(id(&spec, if changed { "replacement" } else { "sparse" }));
        if changed {
            expected.insert(id(&spec, "deleted"));
        }
        assert_eq!(old_ids, expected);
        assert_eq!(new_ids, expected);
        assert!(
            visits > 0,
            "reference-bearing roots must retain the actual traversal"
        );
        old.invalidate_builder_ids(old_ids);
        new.invalidate_builders(&roots);
        same_state(&old, &new);
    }
}

#[test]
fn immutable_reference_memo_invalidates_for_copy_on_write_and_unique_mutation() {
    let spec = spec();
    let files = FxHashMap::default();
    let e = evaluator(&files, FxHashMap::default());
    let mut value = Value::Aggregate(vec![free_tree(&spec)].into());
    let snapshot = value.clone();
    assert!(e
        .collect_builder_ids(&[value.clone()], std::mem::drop)
        .is_empty());
    let Value::Aggregate(values) = &mut value else {
        unreachable!()
    };
    values.push(builder(&spec, "builder"));
    assert_eq!(
        e.collect_builder_ids(&[value.clone()], std::mem::drop),
        FxHashSet::from_iter([id(&spec, "builder")])
    );
    assert!(e
        .collect_builder_ids(&[snapshot], std::mem::drop)
        .is_empty());
    let Value::Aggregate(values) = &mut value else {
        unreachable!()
    };
    values.pop();
    let mut visits = 0;
    assert!(e.collect_builder_ids(&[value], |()| visits += 1).is_empty());
    assert_eq!(visits, 0);
}
