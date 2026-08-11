use super::{evaluation::static_bool, InputState, StaticBool, StaticValue};
use serde_yaml::Value;

#[test]
fn literal_format_conditions_resolve_replacements_and_preserve_unknown_boundaries() {
    let inputs = InputState::from([("label".into(), StaticValue::String("release".into()))]);
    for (expression, expected) in [
        (
            "format('checks-{0}-{1}', inputs.label, 2) == 'checks-release-2'",
            StaticBool::True,
        ),
        (
            "format('{{{0}}}', 'release') == '{release}'",
            StaticBool::True,
        ),
        (
            "format('it''s {0}', 'ready') == 'it''s ready'",
            StaticBool::True,
        ),
        (
            "format('{0}', 1.2345678901234567) == '1.23456789012346'",
            StaticBool::True,
        ),
        ("format('{0}', 2) == '2'", StaticBool::True),
        ("format('{0}', -0) == '-0'", StaticBool::True),
        ("format('{0}', -0) == '0'", StaticBool::False),
        (
            "format('{0}', 1000000000000000) == '1E+15'",
            StaticBool::True,
        ),
        ("format('literal', 'unused') == 'literal'", StaticBool::True),
        ("format('{1}', 'release') == 'release'", StaticBool::Unknown),
        ("format('{0', 'release') == 'release'", StaticBool::Unknown),
        ("format('{0}', 1e9999) == 'Infinity'", StaticBool::Unknown),
        (
            "format('{0}', github.ref) == 'refs/heads/main'",
            StaticBool::Unknown,
        ),
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            expected,
            "{expression}"
        );
    }
}

#[test]
fn literal_format_values_compose_inside_static_string_functions() {
    let inputs = InputState::new();
    for (expression, expected) in [
        ("contains(format('{0}', 'no'), 'yes')", StaticBool::False),
        (
            "startsWith(format('{0}-{1}', 'release', 'candidate'), 'release')",
            StaticBool::True,
        ),
        (
            "endsWith(format('{0}-{1}', 'release', 'candidate'), 'date')",
            StaticBool::True,
        ),
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            expected,
            "{expression}"
        );
    }
}
