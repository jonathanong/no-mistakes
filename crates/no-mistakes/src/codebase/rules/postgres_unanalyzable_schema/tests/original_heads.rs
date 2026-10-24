use super::*;

#[test]
fn original_heads_and_rejected_branch_actions_reach_every_schema_rule() {
    let root = fixture().join("issue-2731");
    let expected: std::collections::BTreeMap<String, Vec<(usize, usize, String)>> =
        serde_json::from_str(&std::fs::read_to_string(root.join("expected.json")).unwrap())
            .unwrap();
    let files: Vec<_> = expected
        .keys()
        .map(|name| root.join("migrations").join(name))
        .collect();
    for (name, rows) in &expected {
        let sql = std::fs::read_to_string(root.join("migrations").join(name)).unwrap();
        let direct = extract_migration_facts(&sql);
        assert_eq!(
            direct
                .unanalyzable_statements
                .iter()
                .map(|failure| { (failure.line, failure.column, failure.head.clone()) })
                .collect::<Vec<_>>(),
            *rows,
            "{name}"
        );
        let statements = crate::codebase::postgres::extract_sql_statement_facts(&sql);
        // Statement facts model executed SQL; dormant function bodies are schema-only.
        let statement_rows: Vec<_> = rows
            .iter()
            .filter(|(line, _, _)| name != "branch-bodies.sql" || *line < 34 || *line > 65)
            .cloned()
            .collect();
        assert_eq!(
            statements
                .parse_failures
                .iter()
                .filter(|failure| failure.schema_unanalyzable)
                .map(|failure| (failure.line, failure.column, failure.head.clone()))
                .collect::<Vec<_>>(),
            statement_rows,
            "{name}"
        );
        if name == "branch-bodies.sql" {
            // Retaining a rejected action must keep each condition's call projection once.
            let mut lines: Vec<_> = direct
                .function_calls
                .iter()
                .filter(|call| call.name_parts == ["abs"])
                .map(|call| call.line)
                .collect();
            lines.sort_unstable();
            assert_eq!(lines, [4, 37]);
        }
    }
    for config_name in ["report.yml", "low.yml"] {
        let config = load_v2_config(&root, Some(&root.join(config_name))).unwrap();
        let facts = crate::codebase::postgres::prepare_rule_sql_facts(
            &root,
            &files,
            source_store_for_files(&files),
            &config,
            RULE_IDS,
        )
        .unwrap();
        let output = diagnostics(&root, &config, &files, &facts).unwrap();
        assert!(output.advisories.is_empty());
        assert_eq!(output.findings.len(), RULE_IDS.len() * 35);
        for rule in RULE_IDS {
            for (name, rows) in &expected {
                let findings: Vec<_> = output
                    .findings
                    .iter()
                    .filter(|finding| {
                        finding.rule == *rule && finding.file == format!("migrations/{name}")
                    })
                    .collect();
                assert_eq!(findings.len(), rows.len(), "{rule}: {name}");
                for (finding, (line, _, head)) in findings.iter().zip(rows) {
                    assert_eq!(finding.line, *line);
                    assert_eq!(finding.target.as_deref(), Some(head.as_str()));
                    assert!(finding
                        .message
                        .contains(&format!("SQL statement `{head}` could not be analyzed")));
                }
            }
        }
    }
}
