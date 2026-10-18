use crate::codebase::postgres::query_annotation::evaluate::{
    frame_arena::ScopeArena, scope::Scope,
};

#[test]
fn unchanged_compaction_keeps_shared_scopes_and_indexes_while_pruning_dead_state() {
    let builder = |id| Value::Prefix("SELECT 1".into(), true, Some(id));
    let mut scopes: ScopeArena = vec![
        Scope::from(FxHashMap::from_iter([
            ("builder".into(), builder(7)),
            ("arguments".into(), Value::Arguments(42)),
        ])),
        Scope::from(FxHashMap::from_iter([
            ("callback".into(), function_value(1)),
            ("builder".into(), builder(8)),
        ])),
        Scope::from(FxHashMap::from_iter([("dead".into(), builder(9))])),
    ]
    .into();
    for (index, id) in [(0, 7), (1, 8)] {
        assert!(scopes[index].contains_builders(&FxHashSet::from_iter([id])));
    }
    let snapshots = [scopes[0].clone(), scopes[1].clone()];
    let returned_values = Values::from(vec![function_value(1)]);
    let returned_snapshot = returned_values.clone();
    let returned = vec![Value::Possible(returned_values)];
    let mut objects = FxHashMap::from_iter([
        (42, vec![function_value(1), Value::Arguments(42)]),
        (99, vec![builder(9)]),
    ]);
    let mut extras = FxHashMap::from_iter([
        (
            42,
            std::collections::BTreeMap::from_iter([(1000, Value::Arguments(42))]),
        ),
        (
            99,
            std::collections::BTreeMap::from_iter([(1000, builder(9))]),
        ),
    ]);
    let mut mapped = FxHashMap::from_iter([
        (1, vec![(42, vec!["parameter".into()])]),
        (2, vec![(99, vec!["dead".into()])]),
    ]);
    let mut fresh = FxHashMap::from_iter([
        (1, FxHashSet::from_iter(["parameter".into()])),
        (2, FxHashSet::from_iter(["dead".into()])),
    ]);
    let mut captured = FxHashMap::from_iter([
        (1, FxHashMap::from_iter([("builder".into(), 0)])),
        (2, FxHashMap::from_iter([("dead".into(), 2)])),
    ]);
    let compacted = compact_from_roots(
        &mut scopes,
        &[0],
        &[],
        &[],
        &returned,
        &mut mapped,
        ArenaMut {
            objects: &mut objects,
            extras: &mut extras,
        },
        &mut fresh,
        &mut captured,
    );
    assert_eq!(compacted.indices, FxHashMap::from_iter([(0, 0), (1, 1)]));
    assert!(
        compacted.captured_changed,
        "dead capture ownership was pruned"
    );
    assert_eq!(compacted.arguments, FxHashSet::from_iter([42]));
    assert_eq!(compacted.identities, FxHashSet::from_iter([7, 8, 42]));
    assert_eq!(
        scopes.ids().collect::<FxHashSet<_>>(),
        FxHashSet::from_iter([0, 1])
    );
    assert!(scopes.get(2).is_none(), "dead handles are released");
    for index in 0..2 {
        assert!(scopes[index].shares_storage_with(&snapshots[index]));
        assert!(scopes[index].builder_index_initialized());
    }
    let Value::Possible(values) = &returned[0] else {
        panic!("returned container disappeared")
    };
    assert_eq!(values.identity(), returned_snapshot.identity());
    assert_eq!(objects.len(), 1);
    assert_eq!(extras.len(), 1);
    assert_eq!(mapped.len(), 1);
    assert_eq!(fresh.len(), 1);
    assert_eq!(captured.len(), 1);
    assert!(objects[&42][1] == Value::Arguments(42));
    assert!(extras[&42][&1000] == Value::Arguments(42));
    assert_eq!(captured[&1]["builder"], 0);
}
