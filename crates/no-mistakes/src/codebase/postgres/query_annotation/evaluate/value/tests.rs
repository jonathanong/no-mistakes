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
