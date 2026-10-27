use super::*;

#[test]
fn live_scope_reads_borrow_only_warm_prefix_only_summaries_without_changing_state() {
    let spec = spec();
    let files = FxHashMap::default();
    let scope = FxHashMap::from_iter([(
        "catalog".into(),
        Value::Aggregate(vec![builder(&spec, "builder"), free_tree(&spec)].into()),
    )]);
    let mut old = evaluator(&files, scope.clone());
    let mut new = evaluator(&files, scope);
    let mut cold = 0;
    let ids = new.collect_live_builder_ids(|()| cold += 1);
    assert_eq!(ids, FxHashSet::from_iter([id(&spec, "builder")]));
    assert!(cold > 0);
    assert!(
        new.scopes[0].cached_references().is_none(),
        "live collection never warms cold scope caches"
    );
    for e in [&old, &new] {
        let edges = e.scopes[0].references();
        assert!(edges.arguments.is_empty());
        assert!(edges.environments.contains(&0));
    }
    let mut warm = 0;
    assert_eq!(new.collect_live_builder_ids(|()| warm += 1), ids);
    assert_eq!(warm, 0);
    old.untrusted_array_intrinsics = true;
    old.invalidate_builder_ids(collect_old(&old, &old.live_builder_values()).0);
    new.invalidate_live_builders();
    same_state(&old, &new);
    assert!(new.untrusted_array_intrinsics);
}

#[test]
fn scope_cache_reuse_preserves_cross_scope_prefix_argument_order_and_fresh_mutable_children() {
    let spec = spec();
    let files = FxHashMap::default();
    let c = id(&spec, "builder");
    let d = id(&spec, "sparse");
    let mut e = evaluator(
        &files,
        FxHashMap::from_iter([("prefix".into(), builder(&spec, "builder"))]),
    );
    e.scopes
        .push(FxHashMap::from_iter([("argument".into(), Value::Arguments(c))]).into());
    e.argument_objects.insert(c, vec![builder(&spec, "sparse")]);
    for scope in &e.scopes {
        scope.references();
    }
    assert_eq!(
        e.collect_live_builder_ids(std::mem::drop),
        FxHashSet::from_iter([c])
    );
    e.scopes.swap(0, 1);
    assert_eq!(
        e.collect_live_builder_ids(std::mem::drop),
        FxHashSet::from_iter([c, d])
    );
    assert_eq!(
        e.collect_live_builder_ids(std::mem::drop),
        collect_old(&e, &e.live_builder_values()).0
    );
    e.argument_objects.get_mut(&c).unwrap()[0] = builder(&spec, "replacement");
    assert_eq!(
        e.collect_live_builder_ids(std::mem::drop),
        FxHashSet::from_iter([c, id(&spec, "replacement")])
    );
    e.argument_extra_slots.insert(
        c,
        std::collections::BTreeMap::from([(9, builder(&spec, "deleted"))]),
    );
    e.definite_deleted_argument_slots.insert((c, 0));
    assert_eq!(
        e.collect_live_builder_ids(std::mem::drop),
        FxHashSet::from_iter([c, id(&spec, "deleted")])
    );
}

#[test]
fn nested_and_direct_scope_writes_invalidate_cached_prefix_only_selection() {
    let spec = spec();
    let files = FxHashMap::default();
    let mut e = evaluator(
        &files,
        FxHashMap::from_iter([(
            "catalog".into(),
            Value::Aggregate(vec![builder(&spec, "builder")].into()),
        )]),
    );
    e.scopes[0].references();
    let snapshot = e.scopes[0].clone();
    let Value::Aggregate(values) = e.scopes[0].get_mut("catalog").unwrap() else {
        unreachable!()
    };
    values.push(builder(&spec, "replacement"));
    assert!(e.scopes[0].cached_references().is_none());
    assert!(!snapshot
        .references()
        .identities
        .contains(&id(&spec, "replacement")));
    assert_eq!(
        e.collect_live_builder_ids(std::mem::drop),
        FxHashSet::from_iter([id(&spec, "builder"), id(&spec, "replacement")])
    );
    e.scopes[0].references();
    e.scopes[0].insert("argument".into(), Value::Array(id(&spec, "argument")));
    e.argument_objects
        .insert(id(&spec, "argument"), vec![builder(&spec, "sparse")]);
    assert!(e.scopes[0].cached_references().is_none());
    e.scopes[0].references();
    assert!(!e.scopes[0].references().arguments.is_empty());
    assert_eq!(
        e.collect_live_builder_ids(std::mem::drop),
        collect_old(&e, &e.live_builder_values()).0
    );
    assert!(e
        .collect_live_builder_ids(std::mem::drop)
        .contains(&id(&spec, "sparse")));
}
