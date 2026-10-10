use super::{Value, Values};
use std::sync::Arc;

fn prefix(text: &str) -> Value {
    Value::Prefix(text.into(), true, None)
}

#[test]
fn cloned_containers_detach_only_the_written_branch() {
    let nested = Values::from(vec![prefix("first"), prefix("second")]);
    let original = Values::from(vec![Value::Aggregate(nested), Value::Primitive]);
    let mut snapshot = original.clone();
    assert!(Arc::ptr_eq(&original.0, &snapshot.0));

    let Value::Aggregate(inner) = &mut snapshot[0] else {
        panic!("the nested container must retain its shape");
    };
    inner[0] = prefix("changed");
    assert!(!Arc::ptr_eq(&original.0, &snapshot.0));
    assert!(original[0] == Value::Aggregate(vec![prefix("first"), prefix("second")].into()));
    assert!(snapshot[0] == Value::Aggregate(vec![prefix("changed"), prefix("second")].into()));
    assert!(original[1] == Value::Primitive);
}

#[test]
fn shared_owned_and_borrowed_iteration_preserves_container_order() {
    let values = Values::from(vec![prefix("first"), prefix("second")]);
    let mut snapshot = values.clone();
    let observed: Vec<_> = (&values).into_iter().cloned().collect();
    assert!(observed == vec![prefix("first"), prefix("second")]);
    for value in &mut snapshot {
        if let Value::Prefix(text, _, _) = value {
            text.push_str(" changed");
        }
    }
    assert!(values.into_iter().collect::<Vec<_>>() == observed);
    assert!(
        snapshot.into_iter().collect::<Vec<_>>()
            == vec![prefix("first changed"), prefix("second changed")]
    );
}

#[test]
fn reference_memo_follows_detached_writes_without_changing_equality() {
    let mut values = Values::from(vec![prefix("plain")]);
    let original = values.clone();
    assert!(!values.contains_reference());
    assert!(values == original);
    values.push(Value::Arguments(4));
    assert!(values.contains_reference());
    assert!(!original.contains_reference());
    assert!(values != original);
    values.pop();
    assert!(!values.contains_reference());
    assert!(values == original);

    let nested = Values::from(vec![Value::Promise(Box::new(Value::Joined(
        vec![Value::Prefix("/* name */ SELECT 1".into(), true, Some(7))].into(),
    )))]);
    assert!(nested.contains_reference());
    let cleared = Values::from(vec![Value::Possible(vec![Value::Unknown].into())]);
    assert!(!cleared.contains_reference());

    let mut outer = Values::from(vec![Value::Aggregate(vec![Value::Primitive].into())]);
    assert!(!outer.contains_reference());
    let Value::Aggregate(inner) = &mut outer[0] else {
        panic!("nested container shape changed");
    };
    inner.push(Value::Arguments(9));
    assert!(outer.contains_reference());
}

#[test]
fn environment_memo_follows_nested_functions_and_detached_writes() {
    use crate::codebase::postgres::query_annotation::Function;
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
    let mut values = Values::from(vec![Value::Arguments(8), prefix("plain")]);
    let original = values.clone();
    assert!(!values.contains_environment());
    values.push(Value::Promise(Box::new(Value::Evaluated(
        Box::new(Value::Joined(
            vec![Value::Possible(
                vec![Value::Aggregate(vec![function].into())].into(),
            )]
            .into(),
        )),
        false,
    ))));
    assert!(values.contains_environment());
    assert_eq!(
        values.environment_indices(),
        &crate::fx::FxHashSet::from_iter([7])
    );
    assert!(!original.contains_environment());
    values.pop();
    assert!(!values.contains_environment());
    assert!(values == original);
}

#[test]
fn prepared_function_summaries_remain_shared_across_expression_snapshots() {
    use crate::codebase::postgres::query_annotation::{Expr, Function};
    let function = Arc::new(Function {
        start: 0,
        params: vec![],
        body: vec![],
        supported: true,
        asynchronous: false,
        arrow: false,
        self_name: None,
    });
    let expression = Expr::Function(function.clone());
    let Expr::Function(snapshot) = expression.clone() else {
        panic!("function shape changed");
    };
    assert!(Arc::ptr_eq(&function, &snapshot));
}
