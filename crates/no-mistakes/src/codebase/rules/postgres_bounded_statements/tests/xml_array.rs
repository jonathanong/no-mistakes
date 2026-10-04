use super::unbounded;

#[test]
fn xml_typed_literals_use_catalog_scalar_proof_without_trusting_custom_types() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/xml-array.sql"
    ));
    assert!(!crate::codebase::postgres::extract_sql_statement_facts(sql).parse_failed);
    assert_eq!(
        unbounded(sql),
        [5, 6, 8].map(|line| ("accounts".to_string(), line))
    );
}
