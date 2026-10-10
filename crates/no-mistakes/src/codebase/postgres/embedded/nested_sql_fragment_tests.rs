use super::{
    extract_embedded_sql_from_source, EmbeddedSqlCall, EmbeddedSqlKind, EmbeddedSqlOptions,
    TrustedSqlTag,
};
use std::path::PathBuf;

fn calls(name: &str) -> Vec<EmbeddedSqlCall> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded")
        .join(name);
    let source = std::fs::read_to_string(&path).expect("fixture");
    let options = EmbeddedSqlOptions::configured("@example/db", &[]).with_trusted_sql_tags(&[
        TrustedSqlTag {
            module: "@example/db".to_string(),
            name: "sql".to_string(),
        },
    ]);
    extract_embedded_sql_from_source(&path, &source, &options).calls
}

fn kinds(name: &str) -> Vec<EmbeddedSqlKind> {
    calls(name).into_iter().map(|call| call.kind).collect()
}

// Before this fix every one of these classified `Inline`, with the spliced
// fragment read as a `sql_placeholder_N` bind value.
#[test]
fn conditional_nested_fragment_is_dynamic() {
    assert_eq!(
        kinds("nested-sql-fragment-conditional.ts"),
        [EmbeddedSqlKind::Dynamic]
    );
}

#[test]
fn unconditional_nested_fragment_is_dynamic() {
    assert_eq!(
        kinds("nested-sql-fragment-direct.ts"),
        [EmbeddedSqlKind::Dynamic]
    );
}

#[test]
fn fragment_bindings_and_aliases_are_dynamic() {
    assert_eq!(
        kinds("nested-sql-fragment-binding.ts"),
        [EmbeddedSqlKind::Dynamic]
    );
}

#[test]
fn tag_helper_calls_logical_arrays_and_appended_fragments_are_dynamic() {
    assert_eq!(
        kinds("nested-sql-fragment-tag-helpers.ts"),
        [EmbeddedSqlKind::Dynamic; 8]
    );
}

#[test]
fn appended_template_with_nested_fragment_is_dynamic() {
    assert_eq!(
        kinds("nested-sql-fragment-append.ts"),
        [EmbeddedSqlKind::Dynamic]
    );
}

#[test]
fn dynamic_nested_fragment_keeps_verified_leading_text() {
    let calls = calls("nested-sql-fragment-direct.ts");
    let sql = calls[0].sql_text.as_deref().expect("leading text");
    assert!(sql.trim_start().starts_with("SELECT id"), "{sql}");
}

// Values, including conditionals between plain values and `String.raw`
// strings, remain parameterized binds and must not be demoted.
#[test]
fn value_interpolations_stay_inline() {
    let calls = calls("nested-sql-value-interpolations.ts");
    assert_eq!(calls.len(), 1, "{calls:?}");
    assert_eq!(calls[0].kind, EmbeddedSqlKind::Inline);
    assert!(
        calls[0]
            .sql_text
            .as_deref()
            .unwrap()
            .contains("kind = sql_placeholder_5"),
        "{calls:?}"
    );
}
