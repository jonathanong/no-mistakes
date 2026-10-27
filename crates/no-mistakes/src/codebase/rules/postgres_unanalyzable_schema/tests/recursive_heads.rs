use super::*;

#[test]
fn recursive_view_heads_match_canonical_facts_messages_and_targets() {
    let root = fixture().join("recursive-heads");
    let sql = std::fs::read_to_string(root.join("migrations/001.sql")).unwrap();
    let direct = extract_migration_facts(&sql).unanalyzable_statements;
    let expected = [
        (2, 1, "CREATE RECURSIVE VIEW top_level AS SELECT 1"),
        (
            3,
            1,
            "CREATE OR REPLACE TEMPORARY RECURSIVE VIEW replacement AS",
        ),
        (4, 9, "CREATE RECURSIVE VIEW malformed (x) AS"),
        (9, 5, "CREATE RECURSIVE VIEW nested AS SELECT 1"),
        (18, 1, "CREATE RECURSIVE VIEW suppressed AS SELECT 1"),
        (19, 1, "CREATE RECURSIVE VIEW"),
        (20, 1, "CREATE RECURSIVE VIEW table_body AS TABLE accounts"),
        (23, 16, "CREATE RECURSIVE VIEW"),
        (25, 16, "CREATE RECURSIVE VIEW guarded_columns AS SELECT 1"),
    ];
    assert_eq!(
        direct
            .iter()
            .map(|failure| (failure.line, failure.column, failure.head.as_str()))
            .collect::<Vec<_>>(),
        expected
    );
    let statements = crate::codebase::postgres::extract_sql_statement_facts(&sql);
    assert_eq!(
        statements
            .parse_failures
            .iter()
            .filter(|failure| failure.schema_unanalyzable)
            .collect::<Vec<_>>(),
        direct.iter().collect::<Vec<_>>()
    );
    // Rejected guarded actions are schema failures at CREATE, not partial IF wrappers.
    let guarded = statements
        .parse_failures
        .iter()
        .filter(|failure| failure.line == 25)
        .collect::<Vec<_>>();
    assert_eq!(guarded.len(), 1);
    assert!(guarded[0].schema_unanalyzable);
    assert_eq!(guarded[0].column, 16);
    for config_name in ["report.yml", "low.yml"] {
        let (config, files, facts) = prepared(&root, config_name);
        assert_eq!(
            facts
                .postgres_schema_file(&files[0])
                .unwrap()
                .unanalyzable_statements,
            direct
        );
        let output = diagnostics(&root, &config, &files, &facts).unwrap();
        // Per-rule messages and lines are in the snapshot. Line 18 exists
        // before suppression and is removed only for postgres-fk-index.
        assert!(output
            .findings
            .iter()
            .any(|finding| { finding.rule == "postgres-fk-index" && finding.line == 18 }));
        let mut findings = output.findings;
        suppress_rule_findings_with_sources(&root, &mut findings, &source_store_for_files(&files));
        assert_eq!(findings.len(), RULE_IDS.len() * (expected.len() - 3) - 1);
        assert_eq!(output.advisories.len(), RULE_IDS.len() * 3);
        assert!(!findings
            .iter()
            .any(|finding| finding.rule == "postgres-fk-index" && finding.line == 18));
    }
}
