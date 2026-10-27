use crate::codebase::postgres::extract_sql_statement_facts;

#[test]
fn bare_boolean_key_predicates_bound_only_complete_conjunctive_keys() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/boolean-keys.sql"
    ));
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 16);
}
