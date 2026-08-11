use super::*;

#[test]
fn literal_environment_values_use_github_string_coercion() {
    for (value, expected) in [
        (Value::Bool(true), StaticValue::String("true".into())),
        (
            Value::Number(serde_yaml::Number::from(42)),
            StaticValue::String("42".into()),
        ),
        (
            Value::String("release".into()),
            StaticValue::String("release".into()),
        ),
        (Value::Null, StaticValue::String(String::new())),
        (Value::Sequence(Vec::new()), StaticValue::Unknown),
        (
            Value::Mapping(serde_yaml::Mapping::new()),
            StaticValue::Unknown,
        ),
    ] {
        assert_eq!(string_value(value), expected);
    }
}
