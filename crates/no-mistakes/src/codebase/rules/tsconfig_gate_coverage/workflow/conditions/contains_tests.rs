use super::{evaluation::static_bool, InputState, StaticBool, StaticValue};
use serde_yaml::Value;

#[test]
fn literal_from_json_arrays_only_resolve_contains_membership() {
    let inputs = InputState::from([
        (
            "\0github.event_name".into(),
            StaticValue::String("schedule".into()),
        ),
        (
            "\0github.event.action".into(),
            StaticValue::String("opened".into()),
        ),
    ]);
    for (expression, expected) in [
        (
            "contains(fromJSON('[\"schedule\"]'), github.event_name)",
            StaticBool::True,
        ),
        (
            "contains(fromJSON('[\"schedule\"]'), github.event.action)",
            StaticBool::False,
        ),
        (
            "contains(fromJSON('[\"SCHEDULE\", \"opened\"]'), github.event.action)",
            StaticBool::True,
        ),
        ("contains(fromJSON('[1]'), '1')", StaticBool::True),
        ("fromJSON('[\"schedule\"]')", StaticBool::TruthyNonBoolean),
        (
            "fromJSON('[\"schedule\"]') == fromJSON('[\"schedule\"]')",
            StaticBool::False,
        ),
        (
            "contains(fromJSON('[[\"schedule\"]]'), github.event_name)",
            StaticBool::False,
        ),
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            expected,
            "{expression}"
        );
    }
}
