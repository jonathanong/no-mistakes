use super::*;

fn job(yaml: &str) -> Value {
    serde_yaml::from_str(yaml).unwrap()
}

#[test]
fn root_matrix_rejects_literal_from_json_values_that_are_not_mappings() {
    for json in ["null", "true", "0", "\"matrix\"", "[]", "[1]"] {
        let expression = format!("strategy:\n  matrix: '${{{{ fromJSON(''{json}'') }}}}'");
        assert!(!matrix_shape_valid(&job(&expression)), "{json}");
    }
}

#[test]
fn root_matrix_expressions_resolve_known_input_json_mappings() {
    let job = job("strategy:\n  matrix: '${{ fromJSON(inputs.matrix) }}'");
    let mut inputs =
        crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::InputState::new();
    inputs.insert(
        "matrix".to_string(),
        crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::StaticValue::String(
            r#"{"project":["resolved"]}"#.to_string(),
        ),
    );

    assert_eq!(
        static_matrix_combinations_for_inputs(&job, &inputs),
        Some(MatrixCombinations::Static(vec![
            std::collections::BTreeMap::from([(
                "project".to_string(),
                Value::String("resolved".to_string()),
            )]),
        ]))
    );
}
