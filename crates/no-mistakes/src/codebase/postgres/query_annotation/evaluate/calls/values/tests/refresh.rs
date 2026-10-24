use super::{legacy, with_evaluator, Value};
use crate::codebase::postgres::query_annotation::evaluate::value::Values;
use crate::fx::{FxHashMap, FxHashSet};

fn saved_dag() -> Value {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/performance/annotation-shared-record-dag/value-equality-distinct/state.json"
    )))
    .unwrap();
    let mut nodes = FxHashMap::<String, Value>::default();
    for node in fixture["nodes"].as_array().unwrap() {
        let value = if let Some(sql) = node["sql"].as_str() {
            Value::Prefix(sql.into(), true, None)
        } else {
            let children = node["children"]
                .as_array()
                .unwrap()
                .iter()
                .map(|name| nodes[name.as_str().unwrap()].clone())
                .collect::<Vec<_>>();
            Value::Record(children.into(), vec!["a".into(), "b".into()])
        };
        nodes.insert(node["name"].as_str().unwrap().into(), value);
    }
    nodes.remove(fixture["left"].as_str().unwrap()).unwrap()
}

fn identities(value: &Value, found: &mut FxHashSet<usize>) {
    if let Value::Record(values, _) = value {
        if found.insert(values.identity() as usize) {
            for value in values {
                identities(value, found);
            }
        }
    }
}

#[test]
fn saved_noop_dag_refresh_preserves_sharing_that_old_refresh_detaches() {
    with_evaluator(|evaluator| {
        let dag = saved_dag();
        let mut expected = vec![dag.clone()];
        let mut actual = expected.clone();
        legacy::refresh(evaluator, &mut expected);
        evaluator.refresh_arguments(&mut actual);
        assert!(actual == expected, "copy-on-write identity is not semantic");
        let mut before = FxHashSet::default();
        let mut after = FxHashSet::default();
        let mut old = FxHashSet::default();
        identities(&dag, &mut before);
        identities(&actual[0], &mut after);
        identities(&expected[0], &mut old);
        assert_eq!(before.len(), 12);
        assert_eq!(before, after, "no-op refresh must borrow every saved node");
        assert_eq!(
            old.len(),
            4095,
            "the frozen refresher expands the shared DAG"
        );
    });
}

fn prefix(id: Option<u64>) -> Value {
    Value::Prefix("/* original */ SELECT 1".into(), true, id)
}
fn cases() -> Vec<Value> {
    let values = || vec![prefix(Some(1)), prefix(Some(2)), prefix(None)].into();
    vec![
        prefix(Some(1)),
        prefix(Some(2)),
        prefix(None),
        Value::Promise(Box::new(prefix(Some(1)))),
        Value::Evaluated(Box::new(prefix(Some(2))), true),
        Value::Aggregate(values()),
        Value::References(values()),
        Value::Object(values()),
        Value::Record(values(), vec!["one".into(), "two".into(), "literal".into()]),
        Value::Joined(values()),
        Value::Possible(values()),
        Value::Arguments(1),
        Value::Array(2),
        Value::Unknown,
        Value::Primitive,
        Value::Number(3),
        Value::Boolean(true),
        Value::Failure("unchanged"),
        Value::Unsupported,
        Value::SlotDeletion,
    ]
}

#[test]
fn updates_invalidations_and_unrelated_histories_match_frozen_semantics() {
    with_evaluator(|evaluator| {
        for (updated, invalidated) in [
            (None, None),
            (Some(1), None),
            (None, Some(2)),
            (Some(1), Some(1)),
            (Some(99), Some(98)),
        ] {
            evaluator.builder_updates.clear();
            evaluator.invalidated_builders.clear();
            if let Some(id) = updated {
                evaluator
                    .builder_updates
                    .insert(id, Value::Prefix("SELECT 2".into(), true, Some(id)));
            }
            if let Some(id) = invalidated {
                evaluator.invalidated_builders.insert(id);
            }
            let mut expected = cases();
            let mut actual = expected.clone();
            legacy::refresh(evaluator, &mut expected);
            evaluator.refresh_arguments(&mut actual);
            assert!(
                actual == expected,
                "updated={updated:?}, invalidated={invalidated:?}"
            );
            if updated == Some(1) && invalidated != Some(1) {
                assert!(
                    matches!(&actual[0], Value::Prefix(sql, true, Some(1)) if sql == "SELECT 2")
                );
            }
            if invalidated == Some(1) {
                assert!(
                    matches!(actual[0], Value::Unknown),
                    "invalidation beats an update"
                );
            }
            assert!(
                matches!(&actual[2], Value::Prefix(sql, true, None) if sql == "/* original */ SELECT 1")
            );
        }
    });
}

#[test]
fn unrelated_builder_work_keeps_reference_free_records_and_wrappers_shared() {
    with_evaluator(|evaluator| {
        evaluator.builder_updates.insert(99, prefix(Some(99)));
        evaluator.invalidated_builders.insert(98);
        let values = Values::from(vec![prefix(None), Value::Number(1)]);
        let mut cases = vec![
            Value::Record(values.clone(), vec!["sql".into(), "count".into()]),
            Value::Promise(Box::new(Value::Aggregate(values.clone()))),
            Value::Evaluated(Box::new(Value::Possible(values.clone())), false),
            Value::References(values.clone()),
            Value::Object(values.clone()),
            Value::Joined(values.clone()),
        ];
        let expected = cases.clone();
        evaluator.refresh_arguments(&mut cases);
        assert!(cases == expected);
        for value in cases {
            let value = match value {
                Value::Promise(inner) | Value::Evaluated(inner, _) => *inner,
                value => value,
            };
            let contents = match value {
                Value::Record(values, _)
                | Value::Aggregate(values)
                | Value::Possible(values)
                | Value::References(values)
                | Value::Object(values)
                | Value::Joined(values) => values,
                _ => unreachable!("saved wrapper cases contain immutable values"),
            };
            assert_eq!(
                contents.identity(),
                values.identity(),
                "borrow before mutable coercion"
            );
        }
    });
}
