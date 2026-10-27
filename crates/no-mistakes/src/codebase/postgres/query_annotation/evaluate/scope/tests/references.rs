use super::super::*;

#[test]
fn reference_memo_shares_snapshots_and_invalidates_both_detached_and_unique_writes() {
    let mut scope = Scope::from(FxHashMap::from_iter([(
        "plain".into(),
        Value::Prefix("SELECT 1".into(), true, None),
    )]));
    let snapshot = scope.clone();
    assert!(!scope.contains_reference());
    assert!(!snapshot.contains_reference());
    assert!(std::ptr::eq(
        scope.0.contains_reference.get().unwrap(),
        snapshot.0.contains_reference.get().unwrap(),
    ));
    assert!(scope == snapshot);
    scope.insert("arguments".into(), Value::Arguments(7));
    assert!(scope.contains_reference());
    assert!(!snapshot.contains_reference());
    assert!(scope != snapshot);
    scope.remove("arguments");
    assert!(!scope.contains_reference());
    for (_, value) in &mut scope {
        *value = Value::Array(8);
    }
    assert!(scope.contains_reference());
    scope.clear();
    assert!(!scope.contains_reference());
    assert!(!Scope::default().contains_reference());
}

#[test]
fn every_prefix_reachable_value_kind_is_conservative() {
    let identified = || Value::Prefix("SELECT 1".into(), true, Some(1));
    let containers = [
        Value::Aggregate(vec![identified()].into()),
        Value::References(vec![identified()].into()),
        Value::Object(vec![identified()].into()),
        Value::Record(vec![identified()].into(), Vec::new()),
        Value::Joined(vec![identified()].into()),
        Value::Possible(vec![identified()].into()),
        Value::Promise(Box::new(identified())),
        Value::Evaluated(Box::new(identified()), false),
        Value::Arguments(9),
        Value::Array(10),
    ];
    for value in containers {
        assert!(value.contains_reference());
        let scope = Scope::from(FxHashMap::from_iter([("value".into(), value)]));
        assert!(scope.contains_reference());
    }
    let function = Value::Function(
        Arc::new(crate::codebase::postgres::query_annotation::Function {
            start: 0,
            params: vec![],
            body: vec![],
            supported: true,
            asynchronous: false,
            arrow: false,
            self_name: None,
        }),
        std::path::PathBuf::from("helper.mts").into(),
        0,
    );
    for value in [
        function,
        Value::Prefix("SELECT 1".into(), true, None),
        Value::Number(1),
        Value::Boolean(false),
        Value::Failure("test"),
        Value::Unknown,
        Value::Primitive,
        Value::Unsupported,
        Value::SlotDeletion,
        Value::Object(vec![Value::Primitive].into()),
    ] {
        assert!(!value.contains_reference());
    }
}
