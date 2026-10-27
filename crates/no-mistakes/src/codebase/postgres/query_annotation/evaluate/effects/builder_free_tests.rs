use super::{
    tests::{collect_old, evaluator, same_state},
    Value,
};
use crate::{
    codebase::postgres::{query_annotation, EmbeddedSqlOptions},
    fx::{FxHashMap, FxHashSet},
};

fn spec() -> serde_json::Value {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("builder-free-values.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}
fn id(spec: &serde_json::Value, key: &str) -> u64 {
    spec[key].as_u64().unwrap()
}
fn builder(spec: &serde_json::Value, key: &str) -> Value {
    Value::Prefix(
        spec["prefix"].as_str().unwrap().into(),
        true,
        Some(id(spec, key)),
    )
}
fn free_tree(spec: &serde_json::Value) -> Value {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("helper-tracing-alternative-callback/src/frame-count.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(
            program,
            &source,
            &EmbeddedSqlOptions::configured("@app/db", &[]),
        )
    })
    .unwrap();
    let function = facts
        .globals
        .values()
        .find_map(|expr| match expr {
            query_annotation::Expr::Function(function) => Some(function.clone()),
            _ => None,
        })
        .unwrap();
    let mut value = Value::Record(
        vec![
            Value::Function(function, path, 0),
            Value::Prefix(spec["prefix"].as_str().unwrap().into(), true, None),
            Value::Number(3),
            Value::Boolean(true),
            Value::Unknown,
        ]
        .into(),
        vec![
            "callback".into(),
            "text".into(),
            "number".into(),
            "boolean".into(),
            "unknown".into(),
        ],
    );
    for _ in 0..id(spec, "depth") {
        value = Value::Aggregate(vec![value; id(spec, "fanout") as usize].into());
    }
    value
}
fn wrappers(value: Value) -> Vec<Value> {
    let values: super::super::value::Values = vec![value].into();
    vec![
        Value::Aggregate(values.clone()),
        Value::References(values.clone()),
        Value::Object(values.clone()),
        Value::Record(values.clone(), vec!["entry".into()]),
        Value::Joined(values.clone()),
        Value::Possible(values.clone()),
        Value::Promise(Box::new(Value::Aggregate(values.clone()))),
        Value::Evaluated(Box::new(Value::Aggregate(values)), true),
    ]
}

// Frozen previous physical-node gate on this builder-free saved input. Mutable
// identities are covered separately by the complete old-ID/state oracle.
fn previous_container_visits(roots: &[Value]) -> usize {
    fn visit(value: &Value, seen: &mut FxHashSet<*const ()>) {
        match value {
            Value::Promise(value) | Value::Evaluated(value, _) => visit(value, seen),
            Value::Aggregate(values)
            | Value::References(values)
            | Value::Object(values)
            | Value::Record(values, _)
            | Value::Joined(values)
            | Value::Possible(values) => {
                if seen.insert(values.identity()) {
                    for value in values {
                        visit(value, seen);
                    }
                }
            }
            _ => {}
        }
    }
    let mut seen = FxHashSet::default();
    for root in roots {
        visit(root, &mut seen);
    }
    seen.len()
}

#[test]
fn builder_free_catalogs_skip_actual_container_visits_cold_and_warm_without_capture_effects() {
    let spec = spec();
    let roots = wrappers(free_tree(&spec));
    let files = FxHashMap::default();
    let scope = FxHashMap::from_iter([("captured builder".into(), builder(&spec, "builder"))]);
    let mut old = evaluator(&files, scope.clone());
    let mut new = evaluator(&files, scope);
    let (old_ids, old_visits) = collect_old(&old, &roots);
    assert_eq!(
        previous_container_visits(&roots),
        id(&spec, "depth") as usize + 2
    );
    assert!(
        old_ids.is_empty(),
        "callbacks never expose captured environments to this collector"
    );
    assert!(
        old_visits > 1000,
        "the saved catalog must exercise the unchanged legacy traversal"
    );
    for _ in 0..2 {
        let mut visits = 0;
        let found = new.collect_builder_ids(&roots, |()| visits += 1);
        assert!(found.is_empty());
        assert_eq!(
            visits, 0,
            "cold and warm builder-free containers bypass the physical-node visitor"
        );
    }
    old.invalidate_builder_ids(old_ids);
    new.invalidate_builders(&roots);
    same_state(&old, &new);
    assert!(matches!(
        new.scopes[0]["captured builder"],
        Value::Prefix(_, _, Some(_))
    ));
    for (index, value) in roots.into_iter().enumerate() {
        old.scopes[0].insert(index.to_string(), value.clone());
        new.scopes[0].insert(index.to_string(), value);
    }
    let roots = old.live_builder_values();
    let (ids, _) = collect_old(&old, &roots);
    old.untrusted_array_intrinsics = true;
    old.invalidate_builder_ids(ids);
    new.invalidate_live_builders();
    same_state(&old, &new);
    assert!(new.invalidated_builders.contains(&id(&spec, "builder")));
}

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
