#[test]
fn three_arms_keep_temporary_and_permanent_namesakes_separate() {
    let strict = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/multi-table-arms.sql"
    ));
    let lenient = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/multi-table-arms-lenient.sql"
    ));
    for (sql, parse_failed) in [(strict, false), (lenient, true)] {
        let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
        assert_eq!(facts.parse_failed, parse_failed);
        assert_eq!(facts.bounds.len(), 1);
    }
}
