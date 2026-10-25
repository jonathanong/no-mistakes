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
    let root = fixture().join("body-policy");
    let (config, files, facts) = prepared(&root, "default.yml");
    let output = diagnostics(&root, &config, &files, &facts).unwrap();
    for rule in RULE_IDS {
        assert_eq!(
            rule_lines(&output.findings, rule),
            [2, 33, 35],
            "{rule}: {:?}",
            output.findings
        );
        assert_eq!(
            rule_lines(&output.advisories, rule),
            [5, 6, 8, 13, 18, 21, 24, 28],
            "{rule}: {:?}",
            output.advisories
        );
    }
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
fn explicit_body_policies_keep_rule_and_repository_precedence() {
    let root = fixture().join("body-policy");
    for (name, findings, advisories) in [
        ("report.yml", 11, 0),
        ("advise.yml", 0, 11),
        ("ignore.yml", 0, 0),
        ("override.yml", 11, 0),
        ("global-alias-report.yml", 11, 0),
        ("rule-advise.yml", 0, 11),
        ("rule-ignore.yml", 0, 0),
    ] {
        let (config, files, facts) = prepared(&root, name);
        let output = diagnostics(&root, &config, &files, &facts).unwrap();
        for rule in RULE_IDS {
            assert_eq!(
                rule_lines(&output.findings, rule).len(),
                findings,
                "{name}: {rule}"
            );
            assert_eq!(
                rule_lines(&output.advisories, rule).len(),
                advisories,
                "{name}: {rule}"
            );
        }
    }
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

#[test]
fn malformed_wrapper_keeps_its_classification_separate_from_an_opaque_child() {
    let root = fixture().join("body-wrapper");
    let config = load_v2_config(&root, Some(&root.join("default.yml"))).unwrap();
    let files = vec![root.join("migrations/001.sql")];
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        &root,
        &files,
        source_store_for_files(&files),
        &config,
        RULE_IDS,
    )
    .unwrap();
    let output = diagnostics(&root, &config, &files, &facts).unwrap();
    for rule in RULE_IDS {
        assert_eq!(rule_lines(&output.findings, rule), [2, 7], "{rule}");
        assert_eq!(rule_lines(&output.advisories, rule), [4], "{rule}");
    }
}
