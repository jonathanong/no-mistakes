use super::super::hooks::measure_live_bindings;
use super::*;

#[test]
fn only_existing_reference_names_skip_false_roots_and_keep_current_argument_contents() {
    let spec = spec();
    let files = FxHashMap::default();
    let count = spec["scopeNameCount"].as_u64().unwrap() as usize;
    let free = free_tree(&spec);
    let argument = id(&spec, "argument");
    let mut bindings = (0..count)
        .map(|i| (format!("scalar_or_callback_{i}"), free.clone()))
        .collect::<FxHashMap<_, _>>();
    bindings.insert("live argument".into(), Value::Arguments(argument));
    let mut old = evaluator(&files, bindings.clone());
    let mut new = evaluator(&files, bindings);
    for e in [&mut old, &mut new] {
        e.argument_objects.insert(
            argument,
            vec![Value::Array(argument), builder(&spec, "builder")],
        );
    }
    assert!(new.scopes[0].cached_reference_bindings().is_none());
    let (cold, visits) = measure_live_bindings(|| new.collect_live_builder_ids(std::mem::drop));
    assert_eq!(cold, FxHashSet::from_iter([argument, id(&spec, "builder")]));
    assert_eq!(visits, count + 1);
    assert!(
        new.scopes[0].cached_reference_bindings().is_none(),
        "membership cannot build or clone a name selection"
    );
    for e in [&old, &new] {
        assert_eq!(e.scopes[0].reference_bindings().count(), 1);
    }
    assert!(new.scopes[0].cached_references().is_none());
    let (warm, visits) = measure_live_bindings(|| new.collect_live_builder_ids(std::mem::drop));
    assert_eq!(warm, cold);
    assert_eq!(visits, 1);
    // An existing graph summary with mutable argument edges must still use the
    // ordered name selection and read the argument arena afresh.
    new.scopes[0].references();
    assert!(!new.scopes[0].references().arguments.is_empty());
    for e in [&mut old, &mut new] {
        e.argument_objects.get_mut(&argument).unwrap()[1] = builder(&spec, "replacement");
        e.argument_extra_slots.insert(
            argument,
            std::collections::BTreeMap::from([(9, builder(&spec, "sparse"))]),
        );
    }
    let (warm, visits) = measure_live_bindings(|| new.collect_live_builder_ids(std::mem::drop));
    assert_eq!(
        warm,
        FxHashSet::from_iter([argument, id(&spec, "replacement"), id(&spec, "sparse")])
    );
    assert_eq!(visits, 1);
    let old_ids = collect_old(&old, &old.live_builder_values()).0;
    assert_eq!(warm, old_ids);
    old.untrusted_array_intrinsics = true;
    old.invalidate_builder_ids(old_ids);
    new.invalidate_live_builders();
    same_state(&old, &new);
    assert!(new.untrusted_array_intrinsics);
}

#[test]
fn cached_reference_names_keep_hash_iteration_order_for_prefix_argument_id_collisions() {
    let spec = spec();
    let files = FxHashMap::default();
    let c = id(&spec, "builder");
    let mut bindings = FxHashMap::from_iter([
        ("prefix".into(), builder(&spec, "builder")),
        ("argument".into(), Value::Arguments(c)),
        ("callback catalog".into(), free_tree(&spec)),
    ]);
    for reverse in [false, true] {
        if reverse {
            let prefix = bindings.remove("prefix").unwrap();
            bindings.insert("last prefix".into(), prefix);
        }
        let mut e = evaluator(&files, bindings.clone());
        e.argument_objects.insert(c, vec![builder(&spec, "sparse")]);
        let original = e.scopes[0]
            .values()
            .filter(|value| value.contains_reference())
            .cloned()
            .collect::<Vec<_>>();
        e.scopes[0].reference_bindings().count();
        let cached = e.scopes[0]
            .cached_reference_bindings()
            .unwrap()
            .cloned()
            .collect::<Vec<_>>();
        assert!(
            original == cached,
            "cached selection must preserve the actual map order"
        );
        assert_eq!(
            e.collect_live_builder_ids(std::mem::drop),
            collect_old(&e, &e.live_builder_values()).0
        );
    }
}

#[test]
fn copy_on_write_and_unique_scope_writes_invalidate_cached_name_selection() {
    let spec = spec();
    let files = FxHashMap::default();
    let mut e = evaluator(
        &files,
        FxHashMap::from_iter([("catalog".into(), free_tree(&spec))]),
    );
    assert_eq!(e.scopes[0].reference_bindings().count(), 0);
    let snapshot = e.scopes[0].clone();
    e.scopes[0].insert("builder".into(), builder(&spec, "builder"));
    assert!(e.scopes[0].cached_reference_bindings().is_none());
    assert_eq!(snapshot.cached_reference_bindings().unwrap().count(), 0);
    assert_eq!(
        e.collect_live_builder_ids(std::mem::drop),
        FxHashSet::from_iter([id(&spec, "builder")])
    );
    assert!(e.scopes[0].cached_reference_bindings().is_none());
    assert_eq!(e.scopes[0].reference_bindings().count(), 1);
    e.scopes[0].remove("builder");
    assert!(e.scopes[0].cached_reference_bindings().is_none());
    assert!(e.collect_live_builder_ids(std::mem::drop).is_empty());
}
