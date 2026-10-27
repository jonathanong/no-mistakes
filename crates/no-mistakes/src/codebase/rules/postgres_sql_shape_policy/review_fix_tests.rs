use super::shape_tests::{config_yaml, fixture, BANNED};
use super::*;

// Output cases moved to cli_postgres_snapshot_sql_shape_policy.

#[test]
fn multiline_placeholder_shapes_keep_the_expression_line_and_suppression() {
    let root = fixture("review-followups");
    let path = root.join("src/interpolation-start.ts");
    let mut findings =
        check_with_files(&root, &config_yaml(BANNED), std::slice::from_ref(&path)).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [3]
    );
    let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty());
}

#[test]
fn implicit_fetch_after_offset_subquery_keeps_each_clause_line_and_suppression() {
    let root = fixture("review-followups");
    let path = root.join("sql/offset-fetch.sql");
    let mut findings = check_with_files(
        &root,
        &config_yaml("sqlInclude: ['sql/**/*.sql']\nbannedShapes: [literal-limit]\nshapeOptions:\n  literalLimit:\n    allowedValues: []\n"),
        std::slice::from_ref(&path),
    )
    .unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [5, 7, 13, 15, 20, 23, 28, 30, 34, 37],
        "{findings:#?}"
    );
    let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [5, 7, 15, 20, 28, 30, 34],
        "Each directive must suppress only its own query's FETCH clause: {findings:#?}"
    );
}

#[test]
fn implicit_fetch_with_unspanned_table_body_keeps_its_clause_line() {
    let root = fixture("review-followups");
    let path = root.join("sql/table-fetch.sql");
    let mut findings = check_with_files(
        &root,
        &config_yaml("sqlInclude: ['sql/**/*.sql']\nbannedShapes: [literal-limit]\nshapeOptions:\n  literalLimit:\n    allowedValues: []\n"),
        std::slice::from_ref(&path),
    )
    .unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [5],
        "{findings:#?}"
    );
    let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty(), "{findings:#?}");
}
