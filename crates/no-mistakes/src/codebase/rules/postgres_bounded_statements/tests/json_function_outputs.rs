#[test]
fn declared_json_output_names_preserve_caller_key_pins() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/json-function-key-pins.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert_eq!(facts.bounds.len(), 7);
}
