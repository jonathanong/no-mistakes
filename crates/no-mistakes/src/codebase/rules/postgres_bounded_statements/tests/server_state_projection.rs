use super::unbounded;
#[test]
fn server_state_select_list_srfs_are_not_sized_by_literal_arguments() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/server-state-projection.sql"));
    assert!(!crate::codebase::postgres::extract_sql_statement_facts(sql).parse_failed);
    assert_eq!(
        unbounded(sql),
        [2, 3, 4].map(|line| ("accounts".to_string(), line))
    );
}
