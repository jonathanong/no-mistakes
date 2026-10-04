#[test]
fn declared_json_output_names_preserve_caller_key_pins() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/json-function-key-pins.sql"));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert_eq!(facts.bounds.len(), 7);
    let catalog = super::catalog();
    let offenders: Vec<Vec<String>> = facts
        .bounds
        .iter()
        .map(|fact| {
            super::offenders(fact, &catalog)
                .into_iter()
                .map(|offender| offender.table)
                .collect()
        })
        .collect();
    assert_eq!(offenders[..5], vec![Vec::<String>::new(); 5]);
    assert_eq!(offenders[5], ["accounts"]);
    assert_eq!(offenders[6], ["orders"]);
}
