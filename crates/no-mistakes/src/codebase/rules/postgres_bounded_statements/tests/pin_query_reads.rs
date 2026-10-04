#[test]
fn pin_query_reads_survive_correlation_and_temporary_shadowing() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/pin-query-read-preservation.sql"));
    assert!(crate::codebase::postgres::parse_postgres_sql(sql).is_ok());
    assert_eq!(
        super::unbounded(sql),
        [
            ("orders".into(), 2),
            ("accounts".into(), 2),
            ("orders".into(), 3),
            ("accounts".into(), 4),
            ("orders".into(), 5),
            ("accounts".into(), 5),
            ("orders".into(), 9),
            ("orders".into(), 11),
            ("orders".into(), 17),
        ]
    );
}

#[test]
fn pin_query_reads_survive_oversized_cte_compaction() {
    let compact = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/compacted-pin-query-reads.sql"));
    assert!(crate::codebase::postgres::parse_postgres_sql(compact).is_ok());
    assert_eq!(
        super::unbounded(compact)
            .iter()
            .map(|(table, _)| table.as_str())
            .collect::<Vec<_>>(),
        ["accounts", "orders", "accounts", "accounts", "orders"]
    );
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
    assert!(findings.iter().any(|finding| finding.line == 17));
    let source = std::fs::read_to_string(&files[1]).unwrap();
    crate::codebase::rules::suppression::suppress_rule_findings_with_source(&mut findings, &source);
    assert!(!findings.iter().any(|finding| finding.line == 17));
    assert!(findings.iter().any(|finding| finding.line == 3));
}

#[test]
fn correlated_pin_dependencies_retire_temporary_views_after_cascade() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temp-view-pin-dependency.sql"));
    assert_eq!(super::unbounded(sql), [("accounts".into(), 4)]);
}
