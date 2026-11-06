use super::check_with_files_and_sources;
use super::shape_tests::{config_yaml, fixture};

#[test]
fn table_page_findings_match_sql_and_embedded_suppression_positions() {
    let root = fixture("review-followups");
    let files = [
        root.join("sql/table-pages.sql"),
        root.join("src/table-pages.ts"),
    ];
    let sources = super::super::source_store_for_files(&files);
    let config =
        config_yaml("sqlInclude: ['sql/table-pages.sql']\nbannedShapes: [keyset-only-sweep]");
    let mut findings = check_with_files_and_sources(&root, &config, &files, &sources).unwrap();
    let lines = |findings: &[super::RuleFinding], suffix: &str| {
        findings
            .iter()
            .filter(|finding| finding.file.ends_with(suffix))
            .map(|finding| finding.line)
            .collect::<Vec<_>>()
    };
    assert_eq!(lines(&findings, "table-pages.sql"), [2, 3, 4, 5, 10, 14]);
    assert_eq!(lines(&findings, "table-pages.ts"), [2, 4]);
    assert!(findings
        .iter()
        .all(|finding| finding.target.as_deref() == Some("keyset-only-sweep")));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert_eq!(lines(&findings, "table-pages.sql"), [2, 3, 4, 5, 10]);
    assert_eq!(lines(&findings, "table-pages.ts"), [2]);
}

#[test]
fn ignored_table_audit_requires_a_complete_applicable_scope() {
    for (scenario, stale) in [
        ("used", false),
        ("stale", true),
        ("overlap", false),
        ("parse-failure", false),
        ("disabled", false),
    ] {
        let root = fixture(&format!("ignore-tables-{scenario}"));
        let config = crate::config::v2::load_v2_config(&root, None).unwrap();
        let files = [root.join("queries.sql")];
        let findings = super::check_with_files(&root, &config, &files).unwrap();
        assert_eq!(
            findings
                .iter()
                .any(|finding| finding.rule == "unused-allow-entry"),
            stale,
            "{scenario}: {findings:#?}"
        );
        assert!(
            scenario == "parse-failure"
                || !findings
                    .iter()
                    .any(|finding| finding.target.as_deref() == Some("keyset-only-sweep")),
            "{scenario}: {findings:#?}"
        );
    }
    let root = fixture("ignore-tables-partial");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let files = [root.join("queries.sql"), root.join("other.sql")];
    let sources = super::super::source_store_for_files(&files);
    let findings = check_with_files_and_sources(&root, &config, &files[..1], &sources).unwrap();
    assert!(
        !findings
            .iter()
            .any(|finding| finding.rule == "unused-allow-entry"),
        "{findings:#?}"
    );
}

#[test]
fn unrecoverable_executed_sql_cannot_prove_ignored_tables_stale() {
    let root = fixture("ignore-tables-unrecoverable");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let findings = super::check_with_files(&root, &config, &[root.join("queries.ts")]).unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
}
