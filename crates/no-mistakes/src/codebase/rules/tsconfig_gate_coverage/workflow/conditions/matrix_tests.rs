use super::evaluation::static_bool;
use super::*;
use serde_yaml::Value;

#[test]
fn missing_matrix_properties_are_empty_strings_but_malformed_access_is_unknown() {
    let no_matrix = InputState::new();
    let missing_axis = InputState::from([("\0matrix.enabled".into(), StaticValue::Bool(true))]);

    for inputs in [&no_matrix, &missing_axis] {
        for (expression, expected) in [
            ("matrix.missing", StaticBool::False),
            ("matrix['missing']", StaticBool::False),
            ("matrix.missing == ''", StaticBool::True),
            ("matrix.missing != ''", StaticBool::False),
            ("contains(matrix.missing, '')", StaticBool::True),
            ("startsWith(matrix.missing, 'value')", StaticBool::False),
        ] {
            assert_eq!(
                static_bool(Some(&Value::String(expression.into())), inputs),
                expected,
                "{expression}"
            );
        }
    }

    assert_eq!(
        static_bool(
            Some(&Value::String("matrix['not valid']".into())),
            &no_matrix,
        ),
        StaticBool::Unknown
    );
}
