use super::*;

#[test]
fn shared_temporary_containers_keep_each_heap_identity_once() {
    let mut value = Value::Possible(
        vec![
            Value::Arguments(8),
            Value::Prefix("SELECT 1".into(), true, Some(7)),
        ]
        .into(),
    );
    // Distinct paths through shared immutable values must not duplicate roots.
    for _ in 0..12 {
        value = Value::Aggregate(vec![value.clone(), value].into());
    }
    let value = Value::Promise(Box::new(Value::Evaluated(Box::new(value), true)));
    let mut roots = Vec::new();
    let mut visited = FxHashSet::default();
    retain(&value, &mut roots, &mut visited);
    assert_eq!(visited.len(), 13);
    assert_eq!(roots.len(), 2);
    assert!(matches!(roots[0], TransientRoot::Argument(8)));
    assert!(matches!(roots[1], TransientRoot::Builder(7)));
}

#[test]
fn scalar_temporary_containers_do_not_allocate_root_entries() {
    let mut value = Value::Possible(
        vec![
            Value::Prefix("SELECT 1".into(), true, None),
            Value::Primitive,
            Value::Unknown,
        ]
        .into(),
    );
    for _ in 0..12 {
        value = Value::Aggregate(vec![value.clone(), value].into());
    }
    let mut roots = Vec::new();
    let mut visited = FxHashSet::default();
    retain(&value, &mut roots, &mut visited);
    assert_eq!(roots.capacity(), 0);
    assert_eq!(visited.capacity(), 0);
}
