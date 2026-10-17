use super::*;
use crate::codebase::postgres::query_annotation::{evaluate::value::Values, Function};
use std::sync::Arc;

#[test]
fn remapping_preserves_shared_values_without_environments() {
    let values = Values::from(vec![Value::Arguments(4)]);
    let identity = values.identity();
    let mut value = Value::Joined(values.clone());
    remap(&mut value, &FxHashMap::from_iter([(7, 2)]));
    let Value::Joined(after) = value else {
        panic!("container shape changed")
    };
    assert_eq!(after.identity(), identity);
}

#[test]
fn remapping_changes_function_environments_only_in_written_snapshot() {
    let function = Value::Function(
        Arc::new(Function {
            start: 0,
            params: vec![],
            body: vec![],
            supported: true,
            asynchronous: false,
            arrow: false,
            self_name: None,
        }),
        "helper.ts".into(),
        7usize.into(),
    );
    let original = Values::from(vec![Value::Promise(Box::new(Value::Evaluated(
        Box::new(function),
        false,
    )))]);
    let mut value = Value::Possible(original.clone());
    remap(&mut value, &FxHashMap::from_iter([(7, 7)]));
    let Value::Possible(before) = &value else {
        panic!("container shape changed")
    };
    assert_eq!(before.identity(), original.identity());
    remap(&mut value, &FxHashMap::from_iter([(7, 7), (9, 2)]));
    let Value::Possible(unaffected) = &value else {
        panic!("container shape changed")
    };
    assert_eq!(unaffected.identity(), original.identity());
    remap(&mut value, &FxHashMap::from_iter([(7, 2)]));
    let Value::Possible(after) = value else {
        panic!("container shape changed")
    };
    assert_ne!(after.identity(), original.identity());
    fn environment(value: &Value) -> usize {
        match value {
            Value::Promise(inner) | Value::Evaluated(inner, _) => environment(inner),
            Value::Function(
                _,
                _,
                crate::codebase::postgres::query_annotation::evaluate::Capture::Lexical(env),
            ) => env.index(),
            _ => panic!("function disappeared"),
        }
    }
    assert_eq!(environment(&after[0]), 2);
    assert_eq!(environment(&original[0]), 7);
}

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
        "helper.ts".into(),
        env.into(),
    )
}

fn function_environment(value: &Value) -> Environment {
    match value {
        Value::Function(
            _,
            _,
            crate::codebase::postgres::query_annotation::evaluate::Capture::Lexical(env),
        ) => env.index(),
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
    let indices = compact_from_roots(
        &mut scopes,
        Roots {
            modules: &Default::default(),
            module_policy: reachable::ModulePolicy::Referenced,
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
    let mut objects = FxHashMap::from_iter([(42, vec![function_value(2), Value::Arguments(42)])]);
    let mut extras = FxHashMap::from_iter([(
        42,
        std::collections::BTreeMap::from_iter([(1000, function_value(2))]),
    )]);
    let mut mapped = FxHashMap::from_iter([(2, vec![(42, vec!["parameter".into()])])]);
    let mut fresh = FxHashMap::from_iter([(2, FxHashSet::from_iter(["parameter".into()]))]);
    let mut captured = FxHashMap::from_iter([(2, FxHashMap::from_iter([("owner".into(), 2)]))]);
    let indices = compact_from_roots(
        &mut scopes,
        Roots {
            modules: &Default::default(),
            module_policy: reachable::ModulePolicy::Referenced,
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
    let mut objects = FxHashMap::from_iter([(42, vec![Value::Arguments(42), function_value(2)])]);
    let mut extras = FxHashMap::default();
    let mut mapped = FxHashMap::default();
    let mut fresh = FxHashMap::default();
    let mut captured = FxHashMap::default();

    let compacted = compact_from_roots(
        &mut scopes,
        Roots {
            modules: &Default::default(),
            module_policy: reachable::ModulePolicy::Referenced,
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

#[test]
fn module_captures_keep_nested_containers_shared_during_numeric_remapping() {
    use crate::codebase::postgres::query_annotation::evaluate::Capture;
    let Value::Function(function, path, _) = function_value(7) else {
        unreachable!()
    };
    let values = Values::from(vec![Value::Promise(Box::new(Value::Function(
        function,
        path,
        Capture::Module,
    )))]);
    assert!(values.contains_environment());
    assert!(values.environment_indices().next().is_none());
    let identity = values.identity();
    let mut wrapped = Value::Object(values);
    remap(&mut wrapped, &FxHashMap::from_iter([(7, 2)]));
    let Value::Object(after) = wrapped else {
        panic!("stored wrapper")
    };
    assert_eq!(after.identity(), identity);
}
