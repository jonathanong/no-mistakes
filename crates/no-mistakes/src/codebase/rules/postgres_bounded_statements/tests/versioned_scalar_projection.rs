use super::unbounded;
#[test]
fn later_builtin_names_need_catalog_identity_without_a_target_version() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/versioned-scalar-projection.sql"));
    assert!(!crate::codebase::postgres::extract_sql_statement_facts(sql).parse_failed);
    assert_eq!(
        unbounded(sql),
        [2, 3, 4, 5, 11, 12, 16].map(|line| ("orders".to_string(), line))
    );
}
