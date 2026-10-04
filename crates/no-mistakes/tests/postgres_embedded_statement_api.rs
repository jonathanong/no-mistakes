use no_mistakes::codebase::postgres::{
    extract_embedded_sql_from_source, extract_sql_statement_facts_for_embedded_call,
    EmbeddedSqlOptions, SqlCursorBound,
};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded")
        .join(name)
}

#[test]
fn public_embedded_call_composition_preserves_placeholder_provenance() {
    let path = fixture("public-statement-facts.ts");
    let source = std::fs::read_to_string(&path).expect("embedded SQL fixture");
    let embedded = extract_embedded_sql_from_source(&path, &source, &EmbeddedSqlOptions::default());

    let recovered = &embedded.calls[0];
    assert_eq!(recovered.recovered_placeholder_positions.len(), 2);
    let facts = extract_sql_statement_facts_for_embedded_call(recovered)
        .expect("the call has recovered SQL");

    let sweep = &facts.sweeps[0];
    assert_eq!(sweep.table, "orders");
    assert_eq!(sweep.order_columns, ["id"]);
    let fake_marker_column = sweep
        .conjuncts
        .iter()
        .find(|conjunct| conjunct.text.contains("fake_id"))
        .expect("the user-authored marker-shaped identifier remains a predicate");
    assert!(fake_marker_column.cursor_columns.is_empty());

    let cursor = sweep
        .conjuncts
        .iter()
        .find(|conjunct| !conjunct.cursor_columns.is_empty())
        .expect("the interpolated cursor bind is recognized");
    assert_eq!(cursor.cursor_columns, ["id"]);
    assert_eq!(cursor.cursor_bound, Some(SqlCursorBound::Lower));

    assert!(extract_sql_statement_facts_for_embedded_call(&embedded.calls[1]).is_none());
}
