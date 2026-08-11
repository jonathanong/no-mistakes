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
