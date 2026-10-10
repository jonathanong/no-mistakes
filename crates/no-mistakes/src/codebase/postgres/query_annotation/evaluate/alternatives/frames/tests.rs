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
        7,
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
            Value::Function(_, _, env) => *env,
            _ => panic!("function disappeared"),
        }
    }
    assert_eq!(environment(&after[0]), 2);
    assert_eq!(environment(&original[0]), 7);
}
