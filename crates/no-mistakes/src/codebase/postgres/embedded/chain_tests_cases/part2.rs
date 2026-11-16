use super::*;

#[test]
fn async_const_bound_arrow_helper_is_rejected() {
    let facts = extract("composed-chain-const-arrow-async.ts");
    assert_eq!(facts.calls[0].kind, super::super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn exported_non_function_declaration_is_ignored_by_helper_collection() {
    let facts = extract("composed-chain-export-class-ignored.ts");
    assert_eq!(facts.calls[0].kind, super::super::EmbeddedSqlKind::Composed);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT id FROM topics WHERE id = 1")
    );
}
