use super::super::function::function_from_definition;

fn source(name: &str) -> String {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/catalog/function-contracts")
        .join(format!("{name}.sql"));
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn saved_function_literals_decode_bodies_without_reading_header_noise() {
    for (name, expected) in [
        ("adjacent", "SELECT 1"),
        ("escape_controls", "SELECT\n\t\r\u{8}\u{c}\\'it's"),
        ("escape_invalid", "xZ|u|U|u12|\u{7}|Ā"),
        ("unicode_long", "SELECT 😀, !"),
        ("unicode_invalid", "SELECT !xyz, +xyz"),
        ("escape_long", "😀"),
        ("atomic_nested", " BEGIN SELECT 1; END; SELECT 2; "),
        ("return_expression", " n + 1\n"),
    ] {
        let definition = source(name);
        let function = function_from_definition("f()", &definition);
        assert_eq!(function.body.as_deref(), Some(expected), "{name}");
        assert_eq!(function.language.as_deref(), Some("sql"), "{name}");
        let (start, end) = function.body_span.expect("source body span");
        assert!(start < end && end <= definition.len(), "{name}");
    }
}

#[test]
fn invalid_or_unclosed_function_literals_leave_the_body_unknown() {
    for name in [
        "unicode_bad_escape",
        "unicode_hex_escape",
        "unicode_space_escape",
        "unclosed_dollar",
        "unclosed_escape",
    ] {
        let function = function_from_definition("f()", &source(name));
        assert_eq!(function.body, None, "{name}");
        assert_eq!(function.body_span, None, "{name}");
        assert_eq!(function.language.as_deref(), Some("sql"), "{name}");
    }
}

#[test]
fn quoted_function_keys_and_planner_settings_preserve_distinct_contracts() {
    let function = function_from_definition(
        "\"odd\"\"(name\"(integer,text)",
        &source("quoted_signature"),
    );
    assert_eq!(function.name, "\"odd\"\"(name\"");
    assert_eq!(function.signature.as_deref(), Some("integer,text"));
    assert!(function.return_contract.contains("out value integer"));
    assert!(function.return_contract.contains("inout label text"));
    assert_eq!(function.body.as_deref(), Some("SELECT 1, 'a'"));
    let planned = function_from_definition("f()", &source("planner_settings"));
    assert_eq!(planned.planner, "support public.estimate cost 3.5 rows 8");
    assert_eq!(planned.parallel, "safe");
    assert_eq!(planned.volatility, "volatile");
    assert_eq!(planned.return_contract, "setof integer");
}

#[test]
fn malformed_unicode_escape_clauses_and_dollar_tags_do_not_invent_a_body() {
    for name in [
        "unicode_empty_escape",
        "unicode_missing_escape",
        "unicode_unquoted_escape",
        "unicode_unclosed_literal",
        "invalid_dollar_tag",
    ] {
        let function = function_from_definition("f()", &source(name));
        assert_eq!(function.body, None, "{name}");
        assert_eq!(function.language.as_deref(), Some("sql"), "{name}");
    }
}

#[test]
fn settings_and_output_types_do_not_capture_unrelated_planner_words() {
    let settings = function_from_definition("f()", &source("set_equals"));
    assert_eq!(settings.planner, "cost 2 support schema.estimate");
    assert_eq!(settings.return_contract, "int");
    let malformed = function_from_definition("f()", &source("malformed_planner"));
    assert!(malformed.planner.is_empty());
    assert_eq!(malformed.body.as_deref(), Some("SELECT 1"));
    let outputs = function_from_definition("f(numeric)", &source("output_types"));
    assert!(outputs.return_contract.contains("out total numeric"));
    assert!(outputs.return_contract.contains("inout tags text"));
    assert_eq!(outputs.body.as_deref(), Some("SELECT 1"));
}
