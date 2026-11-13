use super::{extract_embedded_sql_from_source, EmbeddedSqlCall, EmbeddedSqlOptions, TrustedSqlTag};
use std::path::PathBuf;

fn read_calls(name: &str) -> Vec<EmbeddedSqlCall> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded")
        .join(name);
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@example/db", &[]).with_trusted_sql_tags(&[
        TrustedSqlTag {
            module: "@example/db".into(),
            name: "sql".into(),
        },
    ]);
    extract_embedded_sql_from_source(&path, &source, &options)
        .calls
        .into_iter()
        .filter(|call| call.callee == "read")
        .collect()
}

fn assert_sql(call: &EmbeddedSqlCall, expected: &[&str]) {
    let mut actual: Vec<_> = call
        .variants
        .iter()
        .map(|variant| variant.sql_text.as_str())
        .collect();
    actual.sort_unstable();
    let mut expected = expected.to_vec();
    expected.sort_unstable();
    assert_eq!(actual, expected, "{call:?}");
    assert!(!call.is_unanalyzable(), "{call:?}");
}

fn assert_unknown(call: &EmbeddedSqlCall) {
    assert!(call.is_unanalyzable(), "{call:?}");
    assert!(call.variants.is_empty(), "{call:?}");
}

#[test]
fn template_clauses_keep_independent_choices_shadowing_and_distinct_reference_sites() {
    let calls = read_calls("3304-helper-template-clauses.mts");
    assert_eq!(calls.len(), 21, "{calls:?}");
    assert_sql(&calls[0], &["SELECT 12.5"]);
    assert_sql(&calls[1], &["SELECT id + 2"]);
    assert_sql(&calls[2], &["SELECT 1"]);
    assert_sql(&calls[3], &["SELECT 2"]);
    assert_sql(
        &calls[4],
        &[
            "SELECT id FROM orders",
            "SELECT id FROM accounts",
            "SELECT name FROM orders",
            "SELECT name FROM accounts",
        ],
    );
    assert_sql(&calls[5], &["SELECT created_at", "SELECT updated_at"]);
    for call in &calls[6..9] {
        assert_sql(call, &["SELECT id", "SELECT name"]);
    }
    assert_ne!(calls[7].line, calls[8].line);
    for call in &calls[9..18] {
        assert_unknown(call);
    }
    assert_sql(&calls[18], &["SELECT id"]);
    assert_unknown(&calls[19]);
    assert_unknown(&calls[20]);
}

#[test]
fn trusted_composition_keeps_join_order_and_rejects_incomplete_inputs() {
    let calls = read_calls("3304-helper-template-composition.mts");
    assert_eq!(calls.len(), 18, "{calls:?}");
    assert_sql(&calls[0], &["SELECT 1; SELECT 2"]);
    assert_sql(&calls[1], &[""]);
    assert_sql(&calls[2], &["SELECT 1"]);
    assert_sql(&calls[3], &["SELECT 1"]);
    assert_sql(&calls[4], &["SELECT 1; SELECT 2"]);
    for call in &calls[5..9] {
        assert_unknown(call);
    }
    assert_sql(&calls[9], &["SELECT 1"]);
    assert_sql(&calls[10], &["SELECT 2"]);
    assert_sql(&calls[11], &["SELECT 3"]);
    assert_unknown(&calls[12]);
    assert_unknown(&calls[13]);
    assert_sql(&calls[14], &["SELECT 5"]);
    assert_unknown(&calls[15]);
    assert_sql(&calls[16], &["SELECT sql_placeholder_1, 2"]);
    assert_sql(&calls[17], &["SELECT 1", "SELECT 2"]);
}
