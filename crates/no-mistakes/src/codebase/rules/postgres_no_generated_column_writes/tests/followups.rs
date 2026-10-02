use super::*;

#[test]
fn qualified_alter_generated_columns_and_sql_literal_boundaries_are_preserved() {
    let (_, _, findings) = scan("review-regressions", &["schema.sql", "writes.sql"]);
    let writes: Vec<_> = findings
        .iter()
        .filter(|finding| finding.file == "writes.sql")
        .collect();
    assert_eq!(writes.len(), 8, "{findings:?}");
    assert_eq!(
        writes
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [1, 2, 3, 4, 5, 6, 7, 9]
    );
    assert!(writes[..3]
        .iter()
        .all(|finding| finding.message.contains("trigger-maintained")));
    assert!(writes[3..5]
        .iter()
        .all(|finding| finding.message.contains("generated column")));
    assert!(writes[5..]
        .iter()
        .all(|finding| finding.message.contains("trigger-maintained")));
}

fn scan(name: &str, files: &[&str]) -> (PathBuf, Vec<PathBuf>, Vec<RuleFinding>) {
    let root = unit_fixture(name);
    let paths: Vec<_> = files.iter().map(|file| root.join(file)).collect();
    let findings = check_with_files(
        &root,
        &config_with_options("triggerMaintainedColumns: [updated_at]"),
        &paths,
    )
    .unwrap();
    (root, paths, findings)
}

#[test]
fn alter_columns_are_not_stale_and_keep_positional_insert_order() {
    let (_, _, findings) = scan("alter-trigger", &["schema.sql", "writes.sql"]);
    let findings: Vec<_> = findings
        .into_iter()
        .filter(|finding| finding.target.as_deref() == Some("updated_at"))
        .collect();
    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert!(findings.iter().all(|finding| finding
        .message
        .contains("trigger-maintained column `orders.updated_at`")));
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [1, 2]
    );
}

#[test]
fn generated_definition_wins_across_migrations() {
    let (_, _, findings) = scan("recreated-generated", &["schema.sql", "writes.sql"]);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(findings[0]
        .message
        .contains("do not write generated column"));
}

#[test]
fn raw_sql_occurrences_keep_locations_and_suppress_independently() {
    let (root, paths, mut findings) = scan("write-occurrences", &["schema.sql", "writes.sql"]);
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [3, 6, 7, 8]
    );
    let sources = crate::codebase::rules::source_store_for_files(&paths);
    crate::codebase::rules::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [3, 8]
    );
}

#[test]
fn combined_catalog_reports_both_kinds_in_raw_and_embedded_sql() {
    let (_, _, findings) = scan(
        "both-column-kinds",
        &["schema.sql", "writes.sql", "write.ts"],
    );
    assert_eq!(findings.len(), 4, "{findings:#?}");
    for file in ["writes.sql", "write.ts"] {
        assert_eq!(
            findings
                .iter()
                .filter(|finding| finding.file == file
                    && finding.message.contains("do not write generated column"))
                .count(),
            1
        );
        assert_eq!(
            findings
                .iter()
                .filter(|finding| finding.file == file
                    && finding
                        .message
                        .contains("do not write trigger-maintained column"))
                .count(),
            1
        );
    }
}

#[test]
fn uppercase_generated_assignment_keeps_the_generated_message() {
    let (_, _, findings) = scan("both-column-kinds", &["schema.sql", "upper.sql"]);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(findings[0]
        .message
        .contains("do not write generated column"));
}

#[test]
fn alter_only_table_has_named_writes_but_no_invented_positional_order() {
    let (_, _, findings) = scan("alter-trigger", &["schema.sql", "external.sql"]);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].line, 1);
    assert!(findings[0]
        .message
        .contains("trigger-maintained column `external_orders.updated_at`"));
}

#[test]
fn invalid_schema_glob_preserves_rule_collection_context() {
    let root = unit_fixture("alter-trigger");
    let error = check_with_files(
        &root,
        &config_with_options("sqlInclude: ['[']"),
        &[root.join("schema.sql")],
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("postgres-no-generated-column-writes failed to collect PostgreSQL facts"));
}
