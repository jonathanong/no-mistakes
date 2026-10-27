use super::{hooks, Value};
use crate::codebase::postgres::query_annotation::{evaluate::Scope, Function};
use crate::fx::FxHashMap;
use std::sync::Arc;
mod evaluator;
#[path = "legacy.rs"]
mod legacy;
mod prefix_discriminants;

fn function() -> Value {
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
        std::path::PathBuf::from("helper.mts").into(),
        7,
    )
}
fn cases() -> Vec<Value> {
    let values = || vec![Value::Number(1)].into();
    vec![
        Value::Prefix("SELECT 1".into(), true, None),
        Value::Prefix("SELECT 1".into(), false, None),
        Value::Prefix("SELECT 1".into(), true, Some(1)),
        Value::Prefix("SELECT 2".into(), true, None),
        Value::Promise(Box::new(Value::Number(1))),
        Value::Promise(Box::new(Value::Number(2))),
        Value::Evaluated(Box::new(Value::Number(1)), false),
        Value::Evaluated(Box::new(Value::Number(1)), true),
        Value::Aggregate(values()),
        Value::References(values()),
        Value::Object(values()),
        Value::Record(values(), vec!["a".into(), "b".into()]),
        Value::Record(values(), vec!["b".into(), "a".into()]),
        Value::Joined(values()),
        Value::Possible(values()),
        Value::Arguments(1),
        Value::Arguments(2),
        Value::Array(1),
        Value::Array(2),
        function(),
        Value::Number(1),
        Value::Number(2),
        Value::Boolean(true),
        Value::Boolean(false),
        Value::Failure("one"),
        Value::Failure("two"),
        Value::Unknown,
        Value::Primitive,
        Value::Unsupported,
        Value::SlotDeletion,
    ]
}
#[test]
fn frozen_derived_oracle_matches_all_variant_pairs_and_function_fields() {
    let mut values = cases();
    let Value::Function(summary, path, env) = function() else {
        unreachable!()
    };
    values.push(Value::Function(
        Arc::new((*summary).clone()),
        path.clone(),
        env,
    ));
    values.push(Value::Function(
        summary.clone(),
        std::path::PathBuf::from("different.mts").into(),
        env,
    ));
    values.push(Value::Function(summary.clone(), path.clone(), env + 1));
    let mut changed = (*summary).clone();
    changed.supported = false;
    values.push(Value::Function(Arc::new(changed), path, env));
    for a in &values {
        for b in &values {
            assert_eq!(a == b, legacy::equal(a, b));
            assert_eq!(hooks::measure(true, || a == b).0, legacy::equal(a, b));
        }
    }
    let Value::Function(a, _, _) = &values[19] else {
        unreachable!()
    };
    let Value::Function(b, _, _) = &values[30] else {
        unreachable!()
    };
    assert!(!Arc::ptr_eq(a, b));
    assert!(values[19] == values[30]);
}
#[test]
fn actual_shared_scope_bindings_skip_fallback_and_detached_writes_preserve_equality() {
    let record = || {
        Value::Record(
            vec![Value::Promise(Box::new(Value::Number(1)))].into(),
            vec!["a deliberately long repeated record label".repeat(64); 32],
        )
    };
    let scope = Scope::from(FxHashMap::from_iter([("record".into(), record())]));
    let mut snapshot = scope.clone();
    let a = &scope["record"];
    let b = &snapshot["record"];
    assert!(std::ptr::eq(a, b));
    assert_eq!(hooks::measure(false, || a == b), (true, 0));
    assert!(hooks::measure(true, || a == b).1 > 0);
    let independent = Scope::from(FxHashMap::from_iter([("record".into(), record())]));
    assert!(!std::ptr::eq(a, &independent["record"]));
    let (equal, work) = hooks::measure(false, || a == &independent["record"]);
    assert!(equal && work > 0);
    snapshot.insert("record".into(), Value::Unknown);
    assert!(scope["record"] != snapshot["record"]);
}
#[test]
fn memo_warmth_is_not_semantic_and_cow_does_not_reuse_identity() {
    let mut cold = super::super::Values::from(vec![function(), Value::Arguments(3)]);
    let warm = cold.clone();
    warm.environment_indices();
    warm.contains_reference();
    let a = Value::Record(cold.clone(), vec!["name".into()]);
    let b = Value::Record(warm, vec!["name".into()]);
    assert!(a == b && legacy::equal(&a, &b));
    cold.push(Value::Unknown);
    assert!(a != Value::Record(cold, vec!["name".into()]));
}
