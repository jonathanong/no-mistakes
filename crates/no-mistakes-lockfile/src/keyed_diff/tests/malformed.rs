use super::{diff, fixture, KeyFormat, KeyOptions};

#[test]
fn keyed_json_rejects_invalid_array_entries_and_shape() {
    let options = KeyOptions {
        format: KeyFormat::Json,
        array: Some("fixtures".to_string()),
        id_field: Some("id".to_string()),
    };
    let invalid_id = "JSON array entry needs a nonempty string \"id\"";
    assert_rejected_fixtures(
        &options,
        "json-array",
        "json",
        &[
            ("missing-id", invalid_id),
            ("nonstring-id", invalid_id),
            ("empty-id", invalid_id),
            ("malformed", "duplicate key \"duplicate\""),
            ("invalid-syntax", "invalid JSON"),
            ("not-an-array", "expected a JSON array at \"fixtures\""),
        ],
    );
}

#[test]
fn keyed_diff_rejects_invalid_options_without_a_file() {
    let input = fixture("json-array", "invalid-options", "json");
    let options: Vec<KeyOptions> = serde_json::from_slice(&input).unwrap();
    assert_eq!(options.len(), 4);
    for options in options {
        assert!(diff(&options, None, None).is_err(), "{options:?}");
    }
}

#[test]
fn keyed_strings_decodes_equivalent_escape_forms() {
    let options = KeyOptions {
        format: KeyFormat::Strings,
        array: None,
        id_field: None,
    };
    let escaped = fixture("strings", "escaped-controls", "strings");
    let unicode = fixture("strings", "unicode-controls", "strings");
    assert_eq!(
        diff(&options, None, Some(&escaped)).unwrap().added,
        ["help"]
    );
    assert!(diff(&options, Some(&escaped), Some(&unicode))
        .unwrap()
        .is_empty());
}

#[test]
fn keyed_strings_rejects_unterminated_entries_and_invalid_escapes() {
    let options = KeyOptions {
        format: KeyFormat::Strings,
        array: None,
        id_field: None,
    };
    assert_rejected_fixtures(
        &options,
        "strings",
        "strings",
        &[
            ("unclosed-comment", "unclosed strings comment"),
            ("missing-semicolon", "expected ';' in strings entry"),
            ("unquoted-key", "expected '\"' in strings entry"),
            ("missing-equals", "expected '=' in strings entry"),
            ("duplicate-key", "duplicate key \"alpha\""),
            ("unclosed-inline-comment", "unclosed strings comment"),
            ("incomplete-escape", "incomplete strings escape"),
            ("invalid-hex-escape", "invalid strings Unicode escape"),
            ("non-ascii-octal", "non-ASCII strings octal escape"),
        ],
    );
}

#[test]
fn keyed_strings_rejects_incomplete_utf8_sequences() {
    let options = KeyOptions {
        format: KeyFormat::Strings,
        array: None,
        id_field: None,
    };
    let source = fixture("strings", "moved", "strings");
    let emoji = std::str::from_utf8(&source).unwrap().find('🚀').unwrap();
    assert_eq!(
        diff(&options, None, Some(&source)).unwrap().added,
        ["alpha", "moving", "removed"]
    );
    let error = diff(&options, None, Some(&source[..emoji + 1])).unwrap_err();
    assert!(format!("{error:#}").starts_with("could not parse head content: expected UTF-8 text"));
}

#[test]
fn keyed_resx_rejects_malformed_xml_and_ambiguous_data() {
    let options = KeyOptions {
        format: KeyFormat::Resx,
        array: None,
        id_field: None,
    };
    assert_rejected_fixtures(
        &options,
        "resx",
        "resx",
        &[
            ("malformed", "invalid resx XML"),
            ("wrong-root", "expected a resx root element"),
            ("missing-name", "resx data needs a name attribute"),
            ("missing-value", "resx data needs a value element"),
            ("duplicate-value", "resx data has multiple value elements"),
            ("duplicate-data", "duplicate key \"alpha\""),
            ("nested-value", "resx values must contain text"),
        ],
    );
}

pub(super) fn assert_rejected_fixtures(
    options: &KeyOptions,
    scenario: &str,
    extension: &str,
    cases: &[(&str, &str)],
) {
    let good = fixture(scenario, "base", extension);
    for (name, expected) in cases {
        let malformed = fixture(scenario, name, extension);
        for (base, head, side) in [(&good, &malformed, "head"), (&malformed, &good, "base")] {
            let error = diff(options, Some(base), Some(head)).unwrap_err();
            let diagnostic = format!("{error:#}");
            assert!(
                diagnostic.starts_with(&format!("could not parse {side} content: {expected}")),
                "{scenario}/{name}: {diagnostic}"
            );
        }
    }
}
