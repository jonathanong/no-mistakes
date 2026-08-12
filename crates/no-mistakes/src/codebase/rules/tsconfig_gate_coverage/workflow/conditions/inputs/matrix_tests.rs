use super::super::{evaluation::static_bool, resolution::matrix_property_value, StaticBool};
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
fn static_object_matrix_values_remain_known_truthy_nonstringable_properties() {
    let matrix_values = BTreeMap::from([(
        "cfg".to_string(),
        serde_yaml::from_str("{package: app}").unwrap(),
    )]);
    let inputs = inputs_with_matrix_values(&InputState::new(), &matrix_values, MatrixState::Static);

    assert_eq!(
        matrix_property_value("cfg", &inputs),
        StaticValue::MatrixMapping("\0matrix.cfg".to_string())
    );
    assert_eq!(
        static_bool(
            Some(&Value::String("${{ matrix.cfg == '' }}".into())),
            &inputs
        ),
        StaticBool::False
    );
    assert_eq!(
        static_bool(Some(&Value::String("${{ !matrix.cfg }}".into())), &inputs),
        StaticBool::False
    );
    assert_eq!(
        static_bool(
            Some(&Value::String("${{ matrix.cfg == matrix.cfg }}".into())),
            &inputs
        ),
        StaticBool::True
    );

    let distinct_values = BTreeMap::from([
        (
            "cfg".to_string(),
            serde_yaml::from_str("{package: app}").unwrap(),
        ),
        (
            "other".to_string(),
            serde_yaml::from_str("{package: app}").unwrap(),
        ),
    ]);
    let distinct_inputs =
        inputs_with_matrix_values(&InputState::new(), &distinct_values, MatrixState::Static);
    assert_eq!(
        static_bool(
            Some(&Value::String("${{ matrix.cfg == matrix.other }}".into())),
            &distinct_inputs
        ),
        StaticBool::False
    );
}

#[test]
fn static_matrix_object_properties_preserve_scalar_values_by_path() {
    let matrix_values = BTreeMap::from([(
        "cfg".to_string(),
        serde_yaml::from_str("{enabled: false, nested: {enabled: true, package: app}}").unwrap(),
    )]);
    let inputs = inputs_with_matrix_values(&InputState::new(), &matrix_values, MatrixState::Static);

    assert_eq!(
        static_bool(
            Some(&Value::String("${{ matrix.cfg.enabled == false }}".into())),
            &inputs
        ),
        StaticBool::True
    );
    // Bracket and dot paths must resolve the same nested static mapping value.
    assert_eq!(
        static_bool(
            Some(&Value::String(
                "${{ matrix['cfg'].nested.package == 'app' }}".into()
            )),
            &inputs
        ),
        StaticBool::True
    );
}
