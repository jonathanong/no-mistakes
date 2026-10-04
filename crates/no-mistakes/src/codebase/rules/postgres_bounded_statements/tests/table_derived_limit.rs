fn findings(sql: &str) -> Vec<String> {
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    let catalog = super::catalog();
    facts
        .bounds
        .iter()
        .flat_map(|fact| super::offenders(fact, &catalog))
        .map(|finding| finding.table)
        .collect()
}

#[test]
fn streaming_table_arm_with_derived_limit_is_bounded() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/table-derived-streaming-limit.sql"
    ));
    assert!(findings(sql).is_empty());
}

#[test]
fn blocking_union_reads_table_before_derived_limit() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/table-derived-blocking-limit.sql"
    ));
    assert_eq!(findings(sql), ["accounts"]);
}

#[test]
fn rewritten_derived_table_arms_keep_quoted_identity() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/table-derived-quoted-control.sql"
    ));
    assert_eq!(findings(sql), ["accounts"]);
}
