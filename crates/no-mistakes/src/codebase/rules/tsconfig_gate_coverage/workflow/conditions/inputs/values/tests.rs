use super::*;

#[test]
fn static_expression_values_distinguish_literals_dynamic_values_and_boolean_results() {
    for (expression, expected) in [
        ("${{ true }}", Some(StaticValue::Bool(true))),
        ("${{ null }}", Some(StaticValue::Null)),
        ("${{ fromJSON('[1, 2]') }}", Some(StaticValue::Unknown)),
        (
            "${{ fromJSON('\"release\"') }}",
            Some(StaticValue::String("release".into())),
        ),
        ("${{ fromJSON('null') }}", Some(StaticValue::Null)),
        ("${{ true || false }}", Some(StaticValue::Bool(true))),
        ("${{ 'value' }}", Some(StaticValue::String("value".into()))),
        ("${{ github.ref }}", Some(StaticValue::Unknown)),
        ("${{ }}", None),
    ] {
        assert_eq!(
            static_expression_value(expression, &InputState::new()),
            expected,
            "{expression}"
        );
    }
}

#[test]
fn forwarded_values_cover_event_matrix_and_unavailable_inputs() {
    let parent = InputState::from([
        (
            "\0github.event_name".to_string(),
            StaticValue::String("pull_request".to_string()),
        ),
        (
            "\0github.event.action".to_string(),
            StaticValue::String("opened".to_string()),
        ),
        (
            "\0matrix.target".to_string(),
            StaticValue::String("linux".to_string()),
        ),
        ("value".to_string(), StaticValue::Number("2".to_string())),
    ]);

    for (binding, expected) in [
        (
            "${{ github.event_name }}",
            Some(StaticValue::String("pull_request".to_string())),
        ),
        (
            "${{ github.event.action }}",
            Some(StaticValue::String("opened".to_string())),
        ),
        (
            "${{ matrix.target }}",
            Some(StaticValue::String("linux".to_string())),
        ),
        (
            "${{ inputs.value }}",
            Some(StaticValue::Number("2".to_string())),
        ),
        ("${{ inputs.missing }}", None),
        ("inputs.value", None),
    ] {
        assert_eq!(
            forwarded_input_value(&Value::String(binding.into()), &parent),
            expected
        );
    }
}

#[test]
fn matrix_axis_values_preserve_static_object_values_as_mappings() {
    for (yaml, expected) in [
        ("false", Some(StaticValue::Bool(false))),
        ("2", Some(StaticValue::Number("2".to_string()))),
        ("release", Some(StaticValue::String("release".to_string()))),
        ("null", Some(StaticValue::Null)),
        ("[release]", None),
        ("{target: release}", Some(StaticValue::Mapping)),
    ] {
        let value = serde_yaml::from_str(yaml).expect("valid YAML scalar or structure");
        assert_eq!(matrix_axis_value(&value), expected, "{yaml}");
    }
}
