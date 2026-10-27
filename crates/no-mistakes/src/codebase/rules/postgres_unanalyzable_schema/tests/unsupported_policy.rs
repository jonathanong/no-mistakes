use super::*;

// Output cases moved to cli_postgres_snapshot_unanalyzable_schema.

#[test]
fn default_unsupported_advisories_honor_line_suppressions() {
    let root = fixture().join("unsupported-suppress");
    let (config, files, facts) = prepared(&root, "default.yml");
    let mut output = diagnostics(&root, &config, &files, &facts).unwrap();
    assert!(output.findings.is_empty());
    assert_eq!(output.advisories.len(), 3);
    suppress_rule_findings_with_sources(
        &root,
        &mut output.advisories,
        &source_store_for_files(&files),
    );
    assert_eq!(output.advisories.len(), 1);
    assert_eq!(output.advisories[0].line, 4);
}

#[test]
fn unsupported_recognition_requires_complete_grammar() {
    let sql = std::fs::read_to_string(fixture().join("unsupported-grammar.sql")).unwrap();
    let failures = extract_migration_facts(&sql).unanalyzable_statements;
    // Reject incomplete clauses, qualified role names, and trailing tokens.
    assert_eq!(failures.len(), 18, "{failures:?}");
    for failure in failures {
        assert_eq!(failure.valid_unsupported, failure.line <= 5, "{failure:?}");
    }
}

#[test]
fn default_unsupported_advisories_honor_file_suppression() {
    let root = fixture().join("unsupported-file-suppress");
    let (config, files, facts) = prepared(&root, "default.yml");
    let mut output = diagnostics(&root, &config, &files, &facts).unwrap();
    assert_eq!(output.advisories.len(), 2);
    suppress_rule_findings_with_sources(
        &root,
        &mut output.advisories,
        &source_store_for_files(&files),
    );
    assert!(output.findings.is_empty());
    assert!(output.advisories.is_empty());
}

#[test]
fn unsupported_default_privileges_requires_complete_clauses_and_identifier_lists() {
    let sql = std::fs::read_to_string(fixture().join("unsupported-grammar-clauses.sql")).unwrap();
    let failures = extract_migration_facts(&sql).unanalyzable_statements;
    // Commas require another role/schema name; absent actions and object kinds stay findings.
    assert_eq!(failures.len(), 6, "{failures:?}");
    for failure in failures {
        assert_eq!(failure.valid_unsupported, failure.line == 6, "{failure:?}");
    }
}
