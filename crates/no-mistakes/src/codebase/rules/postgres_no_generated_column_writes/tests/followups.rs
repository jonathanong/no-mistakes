use super::*;

// Output cases moved to cli_postgres_snapshot_generated_column_writes.

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
fn dollar_sign_identifiers_do_not_absorb_later_writes_or_suppressions() {
    let (root, paths, findings) = scan("dollar-identifiers", &["schema.sql", "writes.sql"]);
    // The malformed placeholder remains a schema gap without absorbing later writes.
    let source = std::fs::read_to_string(root.join("writes.sql")).unwrap();
    let invalid = source.lines().nth(4).unwrap().trim_end_matches(';');
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.target.as_deref() != Some("updated_at"))
            .map(|finding| (finding.line, finding.target.as_deref()))
            .collect::<Vec<_>>(),
        [(5, Some(invalid))],
    );
    let mut findings = verified_schema_markers(&root, "writes.sql", findings);
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [2, 4, 6, 7, 8]
    );
    let sources = crate::codebase::rules::source_store_for_files(&paths);
    crate::codebase::rules::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [2, 6, 7, 8]
    );
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
