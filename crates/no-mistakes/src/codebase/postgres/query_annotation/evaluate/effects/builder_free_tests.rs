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
            | Value::Possible(values)
                if seen.insert(values.identity()) =>
            {
                for value in values {
                    visit(value, seen);
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

mod dense_cache;
mod mixed;
mod scope_cache;
mod warm_names;
