use super::*;
use crate::codebase::postgres::query_annotation::evaluate::ArgumentObjects;
use std::collections::BTreeMap;

fn callback(env: usize) -> Value {
    Value::Function(tests::saved_function(), "reads.mts".into(), env)
}

fn compare(
    scopes: &[Scope],
    arena: Arena<'_>,
    mapped: &FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    captured: &CapturedBindings,
    returned: &[Value],
    transient: &[TransientRoot],
    prefix: &mut Prefix,
) -> Reachable {
    let roots = [0];
    let seeds = || RootSeeds {
        environments: &roots,
        transient,
        ..Default::default()
    };
    let old = collect_from_roots(scopes, seeds(), returned, arena, mapped, captured);
    let new = collect_cached(
        scopes,
        seeds(),
        returned,
        arena,
        mapped,
        captured,
        (prefix, 1),
    );
    assert_eq!(old.environments, new.environments);
    assert_eq!(old.arguments, new.arguments);
    assert_eq!(old.identities, new.identities);
    new
}

#[test]
fn unchanged_scope_does_not_freeze_mutable_dense_sparse_or_auxiliary_edges() {
    let mut scopes = vec![
        FxHashMap::from_iter([("args".into(), Value::Arguments(7))]).into(),
        Scope::default(),
        Scope::default(),
        Scope::default(),
        Scope::default(),
    ];
    let mut objects = ArgumentObjects::from_iter([
        (7, vec![Value::Arguments(7)]),
        (8, vec![Value::Arguments(8)]),
    ]);
    let mut extras = FxHashMap::default();
    let mut mapped = FxHashMap::default();
    let mut captured = FxHashMap::default();
    let mut cache = Prefix::default();
    let original = scopes[0].clone();
    compare(
        &scopes,
        Arena {
            objects: &objects,
            extras: &extras,
        },
        &mapped,
        &captured,
        &[],
        &[],
        &mut cache,
    );
    objects.get_mut(&7).unwrap().push(callback(1));
    extras.insert(7, BTreeMap::from([(99, callback(2))]));
    mapped.insert(0, vec![(8, vec!["arg".into()])]);
    captured.insert(0, FxHashMap::from_iter([("captured".into(), 3)]).into());
    let found = compare(
        &scopes,
        Arena {
            objects: &objects,
            extras: &extras,
        },
        &mapped,
        &captured,
        &[callback(4)],
        &[TransientRoot::Builder(19)],
        &mut cache,
    );
    assert_eq!(found.environments, FxHashSet::from_iter(0..5));
    assert_eq!(found.arguments, FxHashSet::from_iter([7, 8]));
    assert!(found.identities.contains(&19));
    assert!(original.same_bindings(&scopes[0]));

    // Removing the last root must drop the cyclic heap and all private frames.
    scopes[0].clear();
    mapped.clear();
    captured.clear();
    let released = compare(
        &scopes,
        Arena {
            objects: &objects,
            extras: &extras,
        },
        &mapped,
        &captured,
        &[],
        &[],
        &mut cache,
    );
    assert_eq!(released.environments, FxHashSet::from_iter([0]));
    assert!(released.arguments.is_empty());
    assert!(released.identities.is_empty());
}

#[test]
fn root_refcounts_survive_duplicates_but_remove_the_last_edge() {
    let mut scopes = vec![
        FxHashMap::from_iter([("a".into(), callback(1)), ("b".into(), callback(1))]).into(),
        Scope::default(),
    ];
    let empty = ArgumentObjects::default();
    let arena = Arena {
        objects: &empty,
        extras: &Default::default(),
    };
    let mut prefix = Prefix::default();
    for name in ["a", "b"] {
        let found = compare(
            &scopes,
            arena,
            &Default::default(),
            &Default::default(),
            &[],
            &[],
            &mut prefix,
        );
        assert!(found.environments.contains(&1));
        scopes[0].remove(name);
    }
    let found = compare(
        &scopes,
        arena,
        &Default::default(),
        &Default::default(),
        &[],
        &[],
        &mut prefix,
    );
    assert!(!found.environments.contains(&1));
}

#[path = "cached_work_tests.rs"]
mod work;
