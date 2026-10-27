use super::super::hooks::measure_live_bindings;
use super::*;

#[test]
fn completed_scope_arguments_allow_zero_root_reads_without_expanding_prefix_collisions() {
    let spec = spec();
    let files = FxHashMap::default();
    let a = id(&spec, "argument");
    let prefix = Value::Prefix("prefix collides with argument".into(), true, Some(a));
    let record = Value::Record(
        vec![Value::Arguments(a), builder(&spec, "builder")].into(),
        vec!["argument".into(), "builder".into()],
    );
    let mut e = evaluator(
        &files,
        FxHashMap::from_iter([("first prefix".into(), prefix)]),
    );
    e.scopes
        .push(FxHashMap::from_iter([("later record".into(), record)]).into());
    e.argument_objects.insert(a, vec![builder(&spec, "sparse")]);
    for scope in &e.scopes {
        scope.references();
    }
    for reverse in [false, true] {
        if reverse {
            e.scopes.swap(0, 1);
        }
        // Use the CURRENT raw value collector: the historical pre-memo oracle
        // does not model shared in-progress immutable containers correctly.
        let raw = e.collect_builder_ids(&e.live_builder_values(), std::mem::drop);
        let (cached, reads) = measure_live_bindings(|| e.collect_live_builder_ids(std::mem::drop));
        let mut expected = FxHashSet::from_iter([a, id(&spec, "builder")]);
        if reverse {
            expected.insert(id(&spec, "sparse"));
        }
        assert_eq!(raw, expected);
        assert_eq!(cached, expected);
        assert_eq!(
            reads,
            usize::from(reverse),
            "unseen first-scope arguments still use fresh DFS"
        );
    }
}

#[test]
fn seen_lexical_arguments_skip_later_scopes_but_first_scope_children_stay_fresh() {
    let spec = spec();
    let files = FxHashMap::default();
    let a = id(&spec, "argument");
    let scope = FxHashMap::from_iter([("arguments".into(), Value::Arguments(a))]);
    let mut old = evaluator(&files, scope.clone());
    let mut new = evaluator(&files, scope);
    for e in [&mut old, &mut new] {
        e.scopes.extend(vec![e.scopes[0].clone(); 16]);
        e.argument_objects
            .insert(a, vec![Value::Arguments(a), builder(&spec, "builder")]);
        for scope in &e.scopes {
            scope.references();
        }
    }
    let (ids, reads) = measure_live_bindings(|| new.collect_live_builder_ids(std::mem::drop));
    assert_eq!(ids, FxHashSet::from_iter([a, id(&spec, "builder")]));
    assert_eq!(reads, 1, "sixteen completed copies need no root reads");
    for e in [&mut old, &mut new] {
        e.argument_objects.get_mut(&a).unwrap()[1] = builder(&spec, "replacement");
        e.argument_extra_slots.insert(
            a,
            std::collections::BTreeMap::from([(9, builder(&spec, "sparse"))]),
        );
    }
    let expected = old.collect_builder_ids(&old.live_builder_values(), std::mem::drop);
    let (ids, reads) = measure_live_bindings(|| new.collect_live_builder_ids(std::mem::drop));
    assert_eq!(
        expected,
        FxHashSet::from_iter([a, id(&spec, "replacement"), id(&spec, "sparse")])
    );
    assert_eq!(ids, expected);
    assert_eq!(reads, 1);
    old.untrusted_array_intrinsics = true;
    old.invalidate_builder_ids(expected);
    new.invalidate_live_builders();
    same_state(&old, &new);
}

#[test]
fn dense_in_progress_arguments_must_not_use_the_completed_scope_guard() {
    let spec = spec();
    let files = FxHashMap::default();
    let a = id(&spec, "argument");
    let b = id(&spec, "array");
    let c = id(&spec, "builder");
    let d = id(&spec, "sparse");
    let shared = Value::Aggregate(vec![Value::Arguments(b), builder(&spec, "builder")].into());
    let mut e = evaluator(&files, FxHashMap::default());
    e.argument_objects
        .insert(b, vec![Value::Arguments(a), Value::Arguments(c)]);
    e.argument_objects.insert(a, vec![shared.clone()]);
    e.argument_objects.insert(c, vec![builder(&spec, "sparse")]);
    let cold = e.collect_builder_ids(std::slice::from_ref(&shared), std::mem::drop);
    assert_eq!(cold, FxHashSet::from_iter([a, b, c, d]));
    for slots in e.argument_objects.values() {
        slots.references();
    }
    // A's cached argument B is already seen but still in progress. Reusing A
    // would insert Prefix(C) early and suppress the argument that reaches D.
    assert!(e.argument_objects[&a].references().arguments.contains(&b));
    let warm = e.collect_builder_ids(&[shared], std::mem::drop);
    assert_eq!(warm, cold);
}
