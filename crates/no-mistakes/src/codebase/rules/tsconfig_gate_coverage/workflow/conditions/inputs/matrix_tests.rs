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
