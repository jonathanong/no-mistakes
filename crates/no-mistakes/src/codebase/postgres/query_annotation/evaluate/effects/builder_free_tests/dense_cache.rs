use super::*;
use std::path::{Path, PathBuf};
type Eval<'a> = crate::codebase::postgres::query_annotation::evaluate::Evaluator<
    'a,
    fn(&str, &Path) -> Option<PathBuf>,
>;

fn collect(e: &Eval<'_>, roots: &[Value]) -> (FxHashSet<u64>, usize) {
    let mut visits = 0;
    let ids = e.collect_builder_ids(roots, |()| visits += 1);
    assert_eq!(ids, collect_old(e, roots).0);
    (ids, visits)
}

#[test]
fn only_warm_prefix_only_dense_edges_skip_containers_without_captured_environment_effects() {
    let spec = spec();
    let files = FxHashMap::default();
    let scope = FxHashMap::from_iter([("captured builder".into(), builder(&spec, "deleted"))]);
    let mut old = evaluator(&files, scope.clone());
    let mut new = evaluator(&files, scope);
    let argument = id(&spec, "argument");
    for e in [&mut old, &mut new] {
        e.argument_objects.insert(
            argument,
            vec![Value::Aggregate(
                vec![builder(&spec, "builder"), free_tree(&spec)].into(),
            )],
        );
    }
    let roots = [Value::Arguments(argument)];
    assert!(new.argument_objects[&argument]
        .cached_references()
        .is_none());
    let (ids, cold) = collect(&new, &roots);
    assert_eq!(ids, FxHashSet::from_iter([argument, id(&spec, "builder")]));
    assert!(cold > 0);
    assert!(
        new.argument_objects[&argument]
            .cached_references()
            .is_none(),
        "membership never initializes cold summaries"
    );
    for e in [&old, &new] {
        let edges = e.argument_objects[&argument].references();
        assert!(edges.arguments.is_empty());
        assert!(edges.environments.contains(&0));
    }
    assert_eq!(collect(&new, &roots), (ids.clone(), 0));
    old.invalidate_builder_ids(ids);
    new.invalidate_builders(&roots);
    same_state(&old, &new);
    assert!(!new.invalidated_builders.contains(&id(&spec, "deleted")));
}

#[test]
fn external_prefix_argument_collisions_keep_original_root_order() {
    let spec = spec();
    let files = FxHashMap::default();
    let mut e = evaluator(&files, FxHashMap::default());
    let a = id(&spec, "argument");
    let c = id(&spec, "builder");
    let d = id(&spec, "sparse");
    e.argument_objects.insert(
        a,
        vec![Value::Aggregate(vec![builder(&spec, "builder")].into())],
    );
    e.argument_objects.insert(c, vec![builder(&spec, "sparse")]);
    e.argument_objects[&a].references();
    for (roots, expected) in [
        (
            vec![
                Value::Prefix("plain".into(), true, Some(a)),
                Value::Arguments(a),
            ],
            FxHashSet::from_iter([a]),
        ),
        (
            vec![
                Value::Arguments(a),
                Value::Prefix("plain".into(), true, Some(a)),
            ],
            FxHashSet::from_iter([a, c]),
        ),
        (
            vec![Value::Arguments(a), Value::Arguments(c)],
            FxHashSet::from_iter([a, c]),
        ),
        (
            vec![Value::Arguments(c), Value::Arguments(a)],
            FxHashSet::from_iter([a, c, d]),
        ),
    ] {
        assert_eq!(collect(&e, &roots).0, expected);
    }
}

#[test]
fn argument_edges_keep_raw_dfs_for_in_progress_shared_container_cycles() {
    let spec = spec();
    let files = FxHashMap::default();
    let mut e = evaluator(&files, FxHashMap::default());
    let a = id(&spec, "argument");
    let c = id(&spec, "builder");
    let d = id(&spec, "sparse");
    // Flattening S into A's summary would insert Prefix(C) too early and lose D.
    let shared = Value::Aggregate(vec![Value::Arguments(a), builder(&spec, "builder")].into());
    e.argument_objects
        .insert(a, vec![shared.clone(), Value::Arguments(c)]);
    e.argument_objects.insert(
        c,
        vec![Value::Aggregate(vec![builder(&spec, "sparse")].into())],
    );
    assert!(!e.argument_objects[&a].references().arguments.is_empty());
    e.argument_objects[&c].references();
    let (ids, visits) = collect(&e, &[shared]);
    assert_eq!(ids, FxHashSet::from_iter([a, c, d]));
    assert!(
        visits > 0,
        "the argument-bearing root must keep the original traversal"
    );
}

#[test]
fn sparse_and_deleted_slots_added_after_warming_keep_the_raw_fallback() {
    let spec = spec();
    let files = FxHashMap::default();
    let mut e = evaluator(&files, FxHashMap::default());
    let a = id(&spec, "argument");
    e.argument_objects.insert(
        a,
        vec![Value::Aggregate(vec![builder(&spec, "builder")].into())],
    );
    e.argument_objects[&a].references();
    let roots = [Value::Array(a)];
    assert_eq!(collect(&e, &roots).1, 0);
    e.argument_extra_slots.insert(
        a,
        std::collections::BTreeMap::from([(8, builder(&spec, "sparse"))]),
    );
    let (ids, visits) = collect(&e, &roots);
    assert_eq!(
        ids,
        FxHashSet::from_iter([a, id(&spec, "builder"), id(&spec, "sparse")])
    );
    assert!(visits > 0);
    e.argument_extra_slots.get_mut(&a).unwrap().clear();
    assert_eq!(collect(&e, &roots).1, 0);
    e.definite_deleted_argument_slots.insert((a, 0));
    assert_eq!(collect(&e, &roots).0, FxHashSet::from_iter([a]));
    e.definite_deleted_argument_slots.clear();
    // Even an unrelated definite deletion conservatively rejects summary reuse.
    e.definite_deleted_argument_slots.insert((999, 0));
    assert!(collect(&e, &roots).1 > 0);
}

#[test]
fn nested_cow_and_unique_dense_writes_invalidate_borrowed_summaries() {
    let spec = spec();
    let files = FxHashMap::default();
    let mut e = evaluator(&files, FxHashMap::default());
    let a = id(&spec, "argument");
    e.argument_objects.insert(
        a,
        vec![Value::Aggregate(vec![builder(&spec, "builder")].into())],
    );
    e.argument_objects[&a].references();
    let snapshot = e.argument_objects[&a].clone();
    {
        let slots = e.argument_objects.get_mut(&a).unwrap();
        let Value::Aggregate(values) = &mut slots[0] else {
            unreachable!()
        };
        values.push(builder(&spec, "replacement"));
    }
    assert!(e.argument_objects[&a].cached_references().is_none());
    assert!(!snapshot
        .references()
        .identities
        .contains(&id(&spec, "replacement")));
    let roots = [Value::Arguments(a)];
    let (ids, visits) = collect(&e, &roots);
    assert_eq!(
        ids,
        FxHashSet::from_iter([a, id(&spec, "builder"), id(&spec, "replacement")])
    );
    assert!(visits > 0);
    e.argument_objects[&a].references();
    assert_eq!(collect(&e, &roots).1, 0);
    {
        let slots = e.argument_objects.get_mut(&a).unwrap();
        let Value::Aggregate(values) = &mut slots[0] else {
            unreachable!()
        };
        values.pop();
    }
    assert!(e.argument_objects[&a].cached_references().is_none());
    assert_eq!(
        collect(&e, &roots).0,
        FxHashSet::from_iter([a, id(&spec, "builder")])
    );
}
