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

#[test]
fn suppression_index_scans_each_line_once_and_matches_existing_queries() {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/ts-source");
    for fixture in ["template-suppression.json", "indexed-suppression.json"] {
        let cases: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(root.join(fixture)).unwrap()).unwrap();
        for case in cases.as_array().unwrap() {
            let source = case["source"].as_str().unwrap();
            let rule = case["rule"].as_str().unwrap_or("template-rule");
            let scans = std::cell::Cell::new(0);
            let index = super::disabled_statement_lines_with_scan_observer(source, rule, || {
                scans.set(scans.get() + 1);
            });
            assert_eq!(index, super::disabled_statement_lines(source, rule));
            let expected_scans = if source.contains("no-mistakes-disable-line ")
                || source.contains("no-mistakes-disable-next-line ")
            {
                source.trim_start_matches('\u{FEFF}').lines().count()
            } else {
                0
            };
            let lines = source.trim_start_matches('\u{FEFF}').lines().count();
            assert_eq!(scans.get(), expected_scans);
            // Queries must consume the index without repeating lexical work.
            for _ in 0..3 {
                for line in 0..=(lines + 2) as u32 {
                    assert_eq!(
                        index.binary_search(&line).is_ok(),
                        super::has_disable_comment(source, line, rule)
                            || super::has_disable_line_comment(source, line, rule),
                        "{fixture}: line {line}: {source}"
                    );
                }
            }
            assert_eq!(scans.get(), expected_scans);
        }
    }
}
