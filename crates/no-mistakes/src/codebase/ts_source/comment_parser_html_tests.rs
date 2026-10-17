use super::{has_disable_comment, has_disable_file_comment, has_disable_line_comment};

#[test]
fn saved_html_and_legacy_directives_keep_lexical_boundaries_and_index_parity() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/ts-source/html-suppression.json");
    let cases: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    for case in cases.as_array().unwrap() {
        let source = case["source"].as_str().unwrap();
        let line = case["line"].as_u64().unwrap() as u32;
        let expected = case["expected"].as_bool().unwrap();
        let kind = case["kind"].as_str().unwrap();
        let actual = match kind {
            "file" => has_disable_file_comment(source, "my-rule"),
            "next" => has_disable_comment(source, line, "my-rule"),
            "line" => has_disable_line_comment(source, line, "my-rule"),
            _ => panic!("unknown saved case kind"),
        };
        assert_eq!(actual, expected, "{case}");
        if kind != "file" {
            let indexed = super::disabled_statement_lines(source, "my-rule");
            assert_eq!(indexed.contains(&line), expected, "index mismatch: {case}");
        }
        let directive = super::matching_disable_directive(source, Some(line), "my-rule");
        let expected_directive = expected.then(|| {
            let line = case["directive_line"].as_u64().unwrap() as u32;
            match kind {
                "file" => super::DisableDirective::File { line },
                "next" => super::DisableDirective::NextLine { line },
                "line" => super::DisableDirective::Line { line },
                _ => panic!("unknown saved directive kind"),
            }
        });
        assert_eq!(directive, expected_directive, "provenance mismatch: {case}");
    }
}
