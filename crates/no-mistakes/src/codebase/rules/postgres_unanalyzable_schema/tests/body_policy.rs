use super::*;

fn rule_lines(items: &[RuleFinding], rule: &str) -> Vec<usize> {
    items
        .iter()
        .filter(|item| item.rule == rule)
        .map(|item| item.line)
        .collect()
}

#[test]
fn default_body_uncertainty_advises_without_downgrading_top_level_errors() {
    // Finding and advisory lines moved to cli_postgres_snapshot_unanalyzable_schema.
    let root = fixture().join("body-policy");
    let (_config, files, facts) = prepared(&root, "default.yml");
    let direct = extract_migration_facts(&std::fs::read_to_string(&files[0]).unwrap());
    let prepared = facts.postgres_schema_file(&files[0]).unwrap();
    assert_eq!(
        direct.unanalyzable_statements,
        prepared.unanalyzable_statements
    );
    for failure in &direct.unanalyzable_statements {
        assert_eq!(
            failure.procedural_body,
            ![2, 33, 35].contains(&failure.line),
            "{failure:?}"
        );
        assert!(
            !failure.valid_unsupported,
            "Body uncertainty does not prove validity: {failure:?}"
        );
    }
    assert!(prepared
        .tables
        .iter()
        .any(|table| table.table_name == "preserved_body_table"));
}

#[test]
fn default_body_advisories_keep_line_and_file_suppressions() {
    let root = fixture().join("body-suppress");
    let (config, mut files, _) = prepared(&root, "default.yml");
    files.push(root.join("migrations/file-suppressed.sql"));
    let sources = source_store_for_files(&files);
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        &root,
        &files,
        sources.clone(),
        &config,
        RULE_IDS,
    )
    .unwrap();
    let mut output = diagnostics(&root, &config, &files, &facts).unwrap();
    assert!(output.findings.is_empty());
    assert_eq!(output.advisories.len(), 4);
    suppress_rule_findings_with_sources(&root, &mut output.advisories, &sources);
    assert_eq!(rule_lines(&output.advisories, "postgres-fk-index"), [6]);
}
