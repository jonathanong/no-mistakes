use super::*;
use crate::codebase::postgres::query_annotation::evaluate::ArgumentObjects;
use crate::codebase::postgres::query_annotation::{evaluate::value::Values, Function};
use std::sync::Arc;

fn function_value(env: Environment) -> Value {
    Value::Function(
        Arc::new(Function {
            start: 0,
            params: vec![],
            body: vec![],
            supported: true,
            asynchronous: false,
            arrow: false,
            self_name: None,
        }),
        std::path::Path::new("helper.ts").into(),
        env,
    )
}

fn function_environment(value: &Value) -> Environment {
    match value {
        Value::Function(_, _, env) => *env,
        Value::Promise(inner) | Value::Evaluated(inner, _) => function_environment(inner),
        _ => panic!("function disappeared"),
    }
}

#[test]
fn unchanged_compaction_keeps_shared_scopes_and_indexes_while_pruning_dead_state() {
    let builder = |id| Value::Prefix("SELECT 1".into(), true, Some(id));
    let mut scopes = vec![
        Scope::from(FxHashMap::from_iter([
            ("builder".into(), builder(7)),
            ("arguments".into(), Value::Arguments(42)),
        ])),
        Scope::from(FxHashMap::from_iter([
            ("callback".into(), function_value(1)),
            ("builder".into(), builder(8)),
        ])),
        Scope::from(FxHashMap::from_iter([("dead".into(), builder(9))])),
    ];
    for (index, id) in [(0, 7), (1, 8)] {
        assert!(scopes[index].contains_builders(&FxHashSet::from_iter([id])));
    }
    let snapshots = scopes[..2].to_vec();
    let returned_values = Values::from(vec![function_value(1)]);
    assert!(returned_values.contains_environment());
    let returned_snapshot = returned_values.clone();
    let mut returned = vec![Value::Possible(returned_values)];
    let mut objects = ArgumentObjects::from_iter([
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
        (1, FxHashMap::from_iter([("builder".into(), 0)]).into()),
        (2, FxHashMap::from_iter([("dead".into(), 2)]).into()),
    ]);
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
    assert_eq!(indices.indices, FxHashMap::from_iter([(0, 0), (1, 1)]));
    assert!(
        indices.captured_changed,
        "dead capture ownership was pruned"
    );
    assert_eq!(indices.arguments, FxHashSet::from_iter([42]));
    assert_eq!(indices.identities, FxHashSet::from_iter([7, 8, 42]));
    assert_eq!(scopes.len(), 2);
    for index in 0..2 {
        // Output equality would miss a needless copy and invalidated memo.
        assert!(std::ptr::eq(&*scopes[index], &*snapshots[index]));
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

#[path = "tests/remapping.rs"]
mod remapping;

mod compaction;
