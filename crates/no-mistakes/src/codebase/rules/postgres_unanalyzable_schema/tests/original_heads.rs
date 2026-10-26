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
            .filter(|(line, _, _)| name != "branch-bodies.sql" || *line < 35 || *line > 66)
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
            for failure in &direct.unanalyzable_statements {
                // #2741 makes body uncertainty advisory by default; the action stays
                // attached to its body even when compatibility parsing rejects it.
                assert_eq!(failure.procedural_body, failure.line < 69, "{failure:?}");
            }
            // Retaining a rejected action must keep each condition's call projection once.
            let mut lines: Vec<_> = direct
                .function_calls
                .iter()
                .filter(|call| call.name_parts == ["abs"])
                .map(|call| call.line)
                .collect();
            lines.sort_unstable();
            assert_eq!(lines, [4, 38]);
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
        let mut findings = output.findings.clone();
        suppress_rule_findings_with_sources(&root, &mut findings, &source_store_for_files(&files));
        assert!(!findings.iter().any(|item| {
            item.rule == "postgres-fk-index"
                && item.file == "migrations/branch-bodies.sql"
                && item.line == 7
        }));
        assert!(findings.iter().any(|item| {
            item.rule == "postgres-constraint-validate"
                && item.file == "migrations/branch-bodies.sql"
                && item.line == 7
        }));
    }
}

#[test]
fn rejected_branch_actions_follow_the_body_default_without_moving_their_heads() {
    let root = fixture().join("issue-2731");
    let config = load_v2_config(&root, Some(&root.join("default.yml"))).unwrap();
    let file = root.join("migrations/branch-bodies.sql");
    let files = vec![file.clone()];
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        &root,
        &files,
        source_store_for_files(&files),
        &config,
        RULE_IDS,
    )
    .unwrap();
    let output = diagnostics(&root, &config, &files, &facts).unwrap();
    let expected: std::collections::BTreeMap<String, Vec<(usize, usize, String)>> =
        serde_json::from_str(&std::fs::read_to_string(root.join("expected.json")).unwrap())
            .unwrap();
    let branch_rows = &expected["branch-bodies.sql"];
    let expected_advice: Vec<_> = branch_rows
        .iter()
        .filter(|(line, _, _)| *line < 69)
        .map(|(line, _, head)| (*line, head.as_str()))
        .collect();
    let actual_advice: Vec<_> = output
        .advisories
        .iter()
        .map(|item| (item.line, item.target.as_deref().unwrap()))
        .collect();
    assert_eq!(actual_advice, expected_advice);
    assert_eq!(
        output
            .findings
            .iter()
            .map(|item| (item.line, item.target.as_deref().unwrap()))
            .collect::<Vec<_>>(),
        vec![(
            69,
            "CREATE RECURSIVE VIEW top_level_after_body AS SELECT 0x1F"
        )]
    );
    let mut advisories = output.advisories.clone();
    suppress_rule_findings_with_sources(&root, &mut advisories, &source_store_for_files(&files));
    assert!(!advisories.iter().any(|item| item.line == 7));
    assert!(advisories.iter().any(|item| item.line == 9));
}
