use super::super::{resolution::matrix_property_value, static_bool, StaticBool};
use super::*;

#[test]
fn forwarding_a_missing_matrix_property_preserves_an_empty_string() {
    let parent = InputState::new();

    assert_eq!(
        values::forwarded_input_value(&Value::String("${{ matrix.missing }}".into()), &parent,),
        Some(StaticValue::String(String::new()))
    );
    assert_eq!(
        values::forwarded_input_value(&Value::String("${{ matrix['not valid'] }}".into()), &parent,),
        None
    );
}

#[test]
fn static_object_matrix_values_remain_known_non_stringable_properties() {
    let matrix_values = BTreeMap::from([(
        "cfg".to_string(),
        serde_yaml::from_str("{package: app}").unwrap(),
    )]);
    let inputs = inputs_with_matrix_values(&InputState::new(), &matrix_values, MatrixState::Static);

    assert_eq!(
        matrix_property_value("cfg", &inputs),
        StaticValue::NonStringable
    );
    assert_eq!(
        static_bool(
            Some(&Value::String("${{ matrix.cfg == '' }}".into())),
            &inputs
        ),
        StaticBool::False
    );
}
