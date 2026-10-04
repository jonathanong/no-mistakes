#[test]
fn literal_null_arguments_preserve_known_fallback_caps() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/nullable-fixed-limit.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 23);
    assert_eq!(facts.limit_uses.len(), 23);
    assert!(facts
        .limit_uses
        .iter()
        .all(|fact| matches!(fact.value, crate::codebase::postgres::SqlLimitValue::Other)));
    let catalog = super::catalog();
    for (index, fact) in facts.bounds.iter().enumerate() {
        assert_eq!(fact.query.capped, index < 10, "statement {index}");
        let names: Vec<String> = super::offenders(fact, &catalog)
            .into_iter()
            .map(|offender| offender.table)
            .collect();
        assert_eq!(
            names,
            if index < 10 {
                vec![]
            } else {
                vec!["accounts".to_string()]
            },
            "statement {index}"
        );
    }
}
