use super::*;
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use std::path::PathBuf;

pub(super) fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/annotation-shared-record-dag/prune-summaries")
}
pub(super) fn saved() -> State {
    let spec: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root().join("state.json")).unwrap()).unwrap();
    let path = root().join("root.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@bench/db", &[]);
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(program, &source, &options)
    })
    .unwrap();
    let function = facts
        .globals
        .iter()
        .find_map(|(_, expr)| {
            if let query_annotation::Expr::Function(function) = expr {
                Some(function.clone())
            } else {
                None
            }
        })
        .expect("saved factory summary");
    let prefix = Value::Prefix(spec["prefix"].as_str().unwrap().into(), true, Some(20));
    let callback = Value::Function(function, path, 0);
    let mut value = Value::Possible(vec![prefix, callback, Value::Arguments(7)].into());
    for _ in 0..6 {
        value = Value::Aggregate(vec![value.clone(), value].into());
    }
    value = Value::Record(
        vec![value].into(),
        vec![spec["label"].as_str().unwrap().into()],
    );
    value = Value::References(
        vec![Value::Object(
            vec![Value::Joined(vec![value].into())].into(),
        )]
        .into(),
    );
    value = Value::Promise(Box::new(Value::Evaluated(Box::new(value), true)));
    let scopes = (0..spec["scopeCount"].as_u64().unwrap())
        .map(|_| {
            (0..spec["bindingsPerScope"].as_u64().unwrap())
                .map(|i| (format!("binding{i}"), value.clone()))
                .collect::<FxHashMap<_, _>>()
                .into()
        })
        .collect();
    let objects = ArgumentObjects::from_iter([
        (7, vec![Value::Arguments(7), Value::Unknown]),
        (8, vec![Value::Arguments(7)]),
        (9, vec![Value::Arguments(9)]),
        (10, vec![Value::Unknown]),
        (99, vec![Value::Unknown]),
    ]);
    State {
        scopes,
        objects,
        extras: FxHashMap::from_iter([(7, BTreeMap::from_iter([(100, Value::Arguments(8))]))]),
        mapped: FxHashMap::from_iter([(0, vec![(9, vec!["mapped".into()])])]),
        captured: FxHashMap::from_iter([(1, FxHashMap::from_iter([("source".into(), 0)]).into())]),
        returned: vec![value],
        transient: vec![
            TransientRoot::Argument(10),
            TransientRoot::Builder(21),
            TransientRoot::Environment(0),
        ],
        deleted: FxHashSet::from_iter([(7, None), (99, Some(0))]),
        definite: FxHashSet::from_iter([(7, 0), (99, 0)]),
        disconnected: FxHashSet::from_iter([(7, 0), (99, 0)]),
        invalidated: FxHashSet::from_iter([20, 21, 99]),
        updates: FxHashMap::from_iter([
            (20, Value::Unknown),
            (21, Value::Unknown),
            (99, Value::Unknown),
        ]),
    }
}
