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
        // Per-rule messages are in the snapshot. This line exists before
        // suppression and is dropped only for postgres-fk-index.
        assert!(output.findings.iter().any(|item| {
            item.rule == "postgres-fk-index"
                && item.file == "migrations/branch-bodies.sql"
                && item.line == 7
        }));
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
    // The full advisory list and the line 69 finding are in the snapshot.
    // Line 7 is present before suppression.
    assert!(output.advisories.iter().any(|item| {
        item.line == 7
            && item.target.as_deref() == Some("CREATE RECURSIVE VIEW do_if_head AS SELECT 1")
    }));
    let mut advisories = output.advisories.clone();
    suppress_rule_findings_with_sources(&root, &mut advisories, &source_store_for_files(&files));
    assert!(!advisories.iter().any(|item| item.line == 7));
    assert!(advisories.iter().any(|item| item.line == 9));
}
