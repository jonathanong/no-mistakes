#[test]
fn copy_payload_quote_does_not_mask_later_quoted_temporary_table() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/copy-quoted-table-identity.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 1);
    assert_eq!(facts.bounds[0].line, 6);
    let catalog = super::catalog();
    let found: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| super::offenders(fact, &catalog))
        .map(|finding| finding.table)
        .collect();
    assert!(
        found.is_empty(),
        "COPY payload must not hide quoted TABLE identity: {found:?}"
    );
}

#[test]
fn copy_payload_quote_preserves_unquoted_permanent_namesake() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/copy-unquoted-table-identity.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 1);
    assert_eq!(facts.bounds[0].line, 6);
    let catalog = super::catalog();
    let found: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| super::offenders(fact, &catalog))
        .map(|finding| finding.table)
        .collect();
    assert_eq!(found, ["accounts"]);
}
