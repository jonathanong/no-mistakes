use super::{tests::fixture, RULE_ID};

#[test]
fn raw_parse_failures_keep_both_statement_lines_and_independent_suppressions() {
    let root = fixture("parse-failure-raw-sites");
    let findings = crate::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    assert!(findings.iter().all(|finding| finding.rule == RULE_ID));
    assert_eq!(
        findings
            .iter()
            .map(|finding| (finding.file.as_str(), finding.line))
            .collect::<Vec<_>>(),
        [
            ("failures.sql", 3),
            ("failures.sql", 6),
            ("one-suppressed.sql", 6),
            ("second-suppressed.sql", 3),
        ],
        "{findings:#?}"
    );
}

#[test]
fn embedded_parse_failures_keep_statement_sites_and_named_suppression() {
    let root = fixture("parse-failure-sites");
    let findings = crate::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| (
                finding.rule.as_str(),
                finding.file.as_str(),
                finding.line,
                finding.target.as_deref(),
            ))
            .collect::<Vec<_>>(),
        [
            (RULE_ID, "failures.test.ts", 4, Some("unanalyzable-sql")),
            (RULE_ID, "failures.test.ts", 7, Some("unanalyzable-sql")),
            (
                RULE_ID,
                "one-suppressed.test.ts",
                7,
                Some("unanalyzable-sql")
            ),
            (
                RULE_ID,
                "second-suppressed.test.ts",
                4,
                Some("unanalyzable-sql")
            ),
        ],
        "{findings:?}"
    );
}
