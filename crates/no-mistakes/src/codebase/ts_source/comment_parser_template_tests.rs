#[test]
fn template_expression_comments_are_code_while_quasis_remain_quoted() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/ts-source/template-suppression.json");
    let cases: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    for case in cases.as_array().unwrap() {
        let source = case["source"].as_str().unwrap();
        let line = case["line"].as_u64().unwrap() as u32;
        let disabled = case["disabled"].as_bool().unwrap();
        assert_eq!(
            super::has_disable_line_comment(source, line, "template-rule"),
            disabled,
            "{source}"
        );
        assert_eq!(
            super::matching_disable_directive(source, Some(line), "template-rule").is_some(),
            disabled,
            "{source}"
        );
    }
}
