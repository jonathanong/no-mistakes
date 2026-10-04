#[test]
fn rejecting_having_preserves_zero_projection_without_hiding_relation_reads() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/rejecting-having-projection.sql"));
    assert!(crate::codebase::postgres::parse_postgres_sql(sql).is_ok());
    assert_eq!(
        super::unbounded(sql),
        [
            ("accounts".into(), 4),
            ("accounts".into(), 5),
            ("accounts".into(), 6),
            ("accounts".into(), 12),
            ("accounts".into(), 13),
            ("accounts".into(), 16),
            ("accounts".into(), 18),
        ]
    );
}

#[test]
fn rejecting_having_findings_keep_statement_locations_and_suppression() {
    let root = super::fixture_root();
    let files = [
        root.join("schema.json"),
        root.join("sql/rejecting-having-projection.sql"),
    ];
    let config = super::config("schemaCatalogPath: schema.json\nsqlInclude: ['sql/rejecting-having-projection.sql']\nexclude: ['src/**']");
    let mut findings = super::check_with_files(&root, &config, &files).unwrap();
    assert!(findings.iter().any(|finding| finding.line == 16));
    let source = std::fs::read_to_string(&files[1]).unwrap();
    crate::codebase::rules::suppression::suppress_rule_findings_with_source(&mut findings, &source);
    assert!(!findings.iter().any(|finding| finding.line == 16));
    assert!(findings.iter().any(|finding| finding.line == 4));
    assert!(!findings
        .iter()
        .any(|finding| [2, 3, 7, 8, 9, 10, 14, 17].contains(&finding.line)));
}
