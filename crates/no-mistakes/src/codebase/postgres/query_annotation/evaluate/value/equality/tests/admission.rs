use super::{legacy, Value};
use crate::codebase::postgres::query_annotation::evaluate::value::Values;

fn outer(value: Value) -> Value {
    // Separate outer allocations prevent the Values identity gateway from
    // skipping the child whose contract this comparison must preserve.
    Value::Aggregate(vec![value].into())
}
fn assert_pair(left: Value, right: Value, expected: bool) {
    assert_eq!(legacy::equal(&left, &right), expected);
    assert_eq!(left == right, expected);
}

#[test]
fn shared_record_content_does_not_replace_its_labels() {
    let values = Values::from(vec![Value::Number(7)]);
    let record = |label: &str| Value::Record(values.clone(), vec![label.into()]);
    assert_pair(outer(record("value")), outer(record("value")), true);
    assert_pair(outer(record("value")), outer(record("other")), false);
}

#[test]
fn shared_reference_and_possible_children_keep_their_variants() {
    let values = Values::from(vec![Value::Number(7)]);
    let wrappers: [fn(Values) -> Value; 2] = [Value::References, Value::Possible];
    for wrap in wrappers {
        assert_pair(
            outer(wrap(values.clone())),
            outer(wrap(values.clone())),
            true,
        );
    }
    assert_pair(
        outer(Value::References(values.clone())),
        outer(Value::Possible(values)),
        false,
    );
}

#[test]
fn nested_deletion_marker_is_not_an_unknown_value() {
    assert_pair(outer(Value::SlotDeletion), outer(Value::SlotDeletion), true);
    assert_pair(outer(Value::SlotDeletion), outer(Value::Unknown), false);
}

fn with_complex_prefix(value: Value) -> Value {
    // Different prefix containers force one Comparison before the later child.
    let prefix = Value::Record(vec![Value::Number(7)].into(), vec!["prefix".into()]);
    Value::Aggregate(vec![prefix, value].into())
}

#[test]
fn active_context_preserves_promise_contents_and_wrapper_variants() {
    let promise = |number| Value::Promise(Box::new(Value::Number(number)));
    assert_pair(
        with_complex_prefix(promise(7)),
        with_complex_prefix(promise(7)),
        true,
    );
    assert_pair(
        with_complex_prefix(promise(7)),
        with_complex_prefix(promise(8)),
        false,
    );
    assert_pair(
        with_complex_prefix(promise(7)),
        with_complex_prefix(Value::Evaluated(Box::new(Value::Number(7)), true)),
        false,
    );
}

#[test]
fn active_context_rejects_different_container_lengths_in_both_orders() {
    let short = || with_complex_prefix(Value::Aggregate(vec![Value::Number(7)].into()));
    let long = || {
        with_complex_prefix(Value::Aggregate(
            vec![Value::Number(7), Value::Number(8)].into(),
        ))
    };
    assert_pair(short(), long(), false);
    assert_pair(long(), short(), false);
}
