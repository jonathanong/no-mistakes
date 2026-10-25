use super::*;

#[test]
fn default_distinguishes_complete_unsupported_grammar_from_malformed_sql() {
    let root = fixture().join("unsupported-policy");
    let (config, files, facts) = prepared(&root, "default.yml");
    let output = diagnostics(&root, &config, &files, &facts).unwrap();
    for rule in RULE_IDS {
        let lines = |items: &[RuleFinding]| {
            items
                .iter()
                .filter(|item| item.rule == *rule)
                .map(|item| item.line)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            lines(&output.findings),
            [4, 5, 6, 7, 6, 7, 8, 9, 10, 11, 12],
            "{rule}"
        );
        assert_eq!(lines(&output.advisories), [1, 2, 3, 2, 3, 4, 5], "{rule}");
    }
}

#[test]
fn explicit_policies_override_both_classes_in_all_schema_rules() {
    let root = fixture().join("unsupported-policy");
    for (name, findings, advisories) in [
        ("report.yml", 18, 0),
        ("advise.yml", 0, 18),
        ("ignore.yml", 0, 0),
        ("override.yml", 18, 0),
        ("global-alias-report.yml", 18, 0),
        ("rule-advise.yml", 0, 18),
        ("rule-ignore.yml", 0, 0),
    ] {
        let (config, files, facts) = prepared(&root, name);
        let output = diagnostics(&root, &config, &files, &facts).unwrap();
        for rule in RULE_IDS {
            assert_eq!(
                output
                    .findings
                    .iter()
                    .filter(|item| item.rule == *rule)
                    .count(),
                findings,
                "{name}: {rule}"
            );
            assert_eq!(
                output
                    .advisories
                    .iter()
                    .filter(|item| item.rule == *rule)
                    .count(),
                advisories,
                "{name}: {rule}"
            );
        }
    }
}

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
