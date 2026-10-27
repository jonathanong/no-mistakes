#[test]
fn pin_query_reads_survive_correlation_and_temporary_shadowing() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/pin-query-read-preservation.sql"));
    assert!(crate::codebase::postgres::parse_postgres_sql(sql).is_ok());
}

#[test]
fn pin_query_reads_survive_oversized_cte_compaction() {
    let compact = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/compacted-pin-query-reads.sql"));
    assert!(crate::codebase::postgres::parse_postgres_sql(compact).is_ok());
}

#[test]
fn retained_pin_read_findings_keep_sql_suppression_locations() {
    let root = super::fixture_root();
    let files = [
        root.join("schema.json"),
        root.join("sql/pin-query-read-preservation.sql"),
    ];
    let config = super::config("schemaCatalogPath: schema.json\nsqlInclude: ['sql/pin-query-read-preservation.sql']\nexclude: ['src/**']");
    let mut findings = super::check_with_files(&root, &config, &files).unwrap();
    // Line 17 is suppressed in check JSON. The pre-suppression finding still names orders.
    assert!(findings
        .iter()
        .any(|finding| { finding.line == 17 && finding.message.contains("orders") }));
    let source = std::fs::read_to_string(&files[1]).unwrap();
    crate::codebase::rules::suppression::suppress_rule_findings_with_source(&mut findings, &source);
    assert!(!findings.iter().any(|finding| finding.line == 17));
    assert!(findings.iter().any(|finding| finding.line == 3));
}
