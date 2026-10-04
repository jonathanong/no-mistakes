use super::unbounded;

#[test]
fn compacted_caller_arrays_retain_type_proofs_without_relation_dependencies() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/oversized-cte-caller-arrays.sql"));
    assert!(!crate::codebase::postgres::extract_sql_statement_facts(sql).parse_failed);
    assert_eq!(
        unbounded(sql),
        [38, 56, 74, 92, 110].map(|line| ("accounts".to_string(), line))
    );
}
