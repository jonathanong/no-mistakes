use super::{static_bool, InputState, StaticBool, StaticValue};
use serde_yaml::Value;

#[test]
fn literal_from_json_arrays_resolve_contains_membership() {
    let inputs = InputState::new();
    for (expression, expected) in [
        (
            "contains(fromJSON('[\"push\", \"schedule\"]'), 'SCHEDULE')",
            StaticBool::True,
        ),
        (
            "contains(fromJSON('[\"push\", \"schedule\"]'), 'workflow_dispatch')",
            StaticBool::False,
        ),
        (
            "contains(fromJSON('[true, 2, null]'), 'true')",
            StaticBool::True,
        ),
        (
            "contains(fromJSON('[true, 2, null]'), '2')",
            StaticBool::True,
        ),
        (
            "contains(fromJSON('[true, 2, null]'), '')",
            StaticBool::True,
        ),
        ("contains(fromJSON('[{}]'), 'x')", StaticBool::False),
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            expected,
            "{expression}"
        );
    }
}

#[test]
fn literal_from_json_arrays_are_distinct_for_equality() {
    let inputs = InputState::new();
    for (expression, expected) in [
        ("fromJSON('[]') == fromJSON('[]')", StaticBool::False),
        ("fromJSON('[]') != fromJSON('[]')", StaticBool::True),
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            expected,
            "{expression}"
        );
    }
}

#[test]
fn unknown_sequence_members_remain_conservative() {
    let inputs = InputState::from([(
        "values".into(),
        StaticValue::Sequence(vec![StaticValue::Unknown]),
    )]);
    assert_eq!(
        static_bool(
            Some(&Value::String("contains(inputs.values, 'x')".into())),
            &inputs
        ),
        StaticBool::Unknown
    );
}
