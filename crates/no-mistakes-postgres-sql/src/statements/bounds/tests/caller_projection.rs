use super::{extract_sql_statement_facts, SqlBoundItemKind};

#[test]
fn output_provenance_retains_values_separately_from_srf_rows() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/caller-projection-keys.sql"));
    let facts = extract_sql_statement_facts(sql);
    let SqlBoundItemKind::Query(query) = &facts.bounds[0].query.items[1].kind else {
        panic!("derived query expected");
    };
    assert_eq!(
        query
            .outputs
            .iter()
            .map(|output| (output.name.as_deref(), output.caller_sized))
            .collect::<Vec<_>>(),
        [(Some("id"), true), (Some("entry"), false)]
    );
    assert!(!query.capped);
    assert!(query
        .items
        .iter()
        .any(|item| matches!(item.kind, SqlBoundItemKind::Opaque)));
}
