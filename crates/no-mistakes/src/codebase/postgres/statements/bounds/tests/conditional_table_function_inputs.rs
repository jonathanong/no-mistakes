use crate::codebase::postgres::SqlBoundItemKind;

#[test]
fn caller_conditional_inputs_preserve_from_and_projection_cardinality() {
    let facts = super::facts(include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/conditional-table-function-inputs.sql")));
    assert_eq!(facts.len(), 21);
    for (index, fact) in facts.iter().enumerate() {
        let opaque = fact
            .query
            .items
            .iter()
            .any(|item| matches!(item.kind, SqlBoundItemKind::Opaque));
        assert_eq!(
            opaque,
            [5, 6, 7, 8, 9, 10, 11, 14, 15, 17, 18, 19, 20].contains(&index),
            "index {index}: {fact:?}"
        );
    }
}
