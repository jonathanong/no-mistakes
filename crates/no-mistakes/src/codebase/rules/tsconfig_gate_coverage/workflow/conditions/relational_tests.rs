use super::{static_bool, InputState, StaticBool, StaticValue};
use serde_yaml::Value;

#[test]
fn relational_comparisons_coerce_known_values_and_keep_dynamic_unknown() {
    let inputs = InputState::from([
        ("numeric_string".into(), StaticValue::String("2".into())),
        (
            "invalid_string".into(),
            StaticValue::String("release".into()),
        ),
        ("empty".into(), StaticValue::String(String::new())),
        ("enabled".into(), StaticValue::Bool(true)),
        ("disabled".into(), StaticValue::Bool(false)),
        ("invalid_number".into(), StaticValue::Number("NaN".into())),
        ("dynamic".into(), StaticValue::Unknown),
    ]);

    for (expression, expected) in [
        ("inputs.numeric_string < 3", StaticBool::True),
        ("inputs.numeric_string <= 2", StaticBool::True),
        ("inputs.numeric_string > 1", StaticBool::True),
        ("inputs.numeric_string >= 2", StaticBool::True),
        ("inputs.invalid_string < 0", StaticBool::False),
        ("inputs.invalid_string <= 0", StaticBool::False),
        ("inputs.invalid_string > 0", StaticBool::False),
        ("inputs.invalid_string >= 0", StaticBool::False),
        ("inputs.empty <= 0", StaticBool::True),
        ("null < 1", StaticBool::True),
        ("inputs.enabled > 0", StaticBool::True),
        ("inputs.disabled >= 0", StaticBool::True),
        ("inputs.invalid_number >= 0", StaticBool::False),
        ("inputs.dynamic > 0", StaticBool::Unknown),
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            expected,
            "{expression}"
        );
    }

    assert_eq!(
        static_bool(
            Some(&Value::String("inputs.numeric_string == 2".into())),
            &inputs,
        ),
        StaticBool::True
    );
    assert_eq!(
        static_bool(
            Some(&Value::String("inputs.invalid_string == 0".into())),
            &inputs,
        ),
        StaticBool::False
    );
}
