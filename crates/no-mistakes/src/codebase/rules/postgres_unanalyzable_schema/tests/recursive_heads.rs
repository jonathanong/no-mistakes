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
        (4, 9, "CREATE RECURSIVE VIEW malformed ( x ) AS"),
        (9, 5, "CREATE RECURSIVE VIEW nested AS SELECT 1"),
        (18, 1, "CREATE RECURSIVE VIEW suppressed AS SELECT 1"),
        (19, 1, "CREATE RECURSIVE VIEW"),
        (20, 1, "CREATE RECURSIVE VIEW table_body AS TABLE accounts"),
        (23, 16, "CREATE RECURSIVE VIEW"),
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
    // A surviving IF condition keeps its partial failure in the statement projection only.
    let partial = statements
        .parse_failures
        .iter()
        .find(|failure| failure.line == 25)
        .unwrap();
    assert_eq!(partial.column, 3);
    assert_eq!(
        partial.head,
        "IF TRUE THEN CREATE RECURSIVE VIEW guarded_columns AS"
    );
    assert!(!partial.schema_unanalyzable);
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
        assert!(output.advisories.is_empty());
        for rule_id in RULE_IDS {
            let findings = output
                .findings
                .iter()
                .filter(|finding| finding.rule == *rule_id)
                .collect::<Vec<_>>();
            assert_eq!(
                findings.len(),
                expected.len(),
                "{config_name} {rule_id}: {findings:?}"
            );
            for (finding, (line, _, head)) in findings.into_iter().zip(expected) {
                assert_eq!(finding.line, line);
                assert_eq!(finding.target.as_deref(), Some(head));
                assert!(
                    finding
                        .message
                        .contains(&format!("SQL statement `{head}` could not be analyzed")),
                    "{finding:?}"
                );
            }
        }
        let mut findings = output.findings;
        suppress_rule_findings_with_sources(&root, &mut findings, &source_store_for_files(&files));
        assert_eq!(findings.len(), RULE_IDS.len() * expected.len() - 1);
        assert!(!findings
            .iter()
            .any(|finding| finding.rule == "postgres-fk-index" && finding.line == 18));
    }
}
