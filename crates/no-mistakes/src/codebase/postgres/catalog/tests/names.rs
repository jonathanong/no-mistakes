#[test]
fn quoted_identifier_fixtures_preserve_case_and_component_boundaries() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/catalog/identifier-names.json");
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    for case in cases {
        let input = case["input"].as_str().unwrap();
        assert_eq!(
            super::super::names::normalize_table_name(input),
            case["table"].as_str().unwrap()
        );
        assert_eq!(
            super::super::names::normalize_identifier(input),
            case["identifier"].as_str().unwrap()
        );
    }
}
