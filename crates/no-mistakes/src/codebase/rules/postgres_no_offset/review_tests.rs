use super::tests::{config_with_options, fixture};
use super::*;

// Output cases moved to cli_postgres_snapshot_no_offset. Keep the private offset
// classification and the pre-suppression lines below.

#[test]
fn non_analyzing_explain_does_not_execute_the_offset_query() {
    let root = fixture("review-followups");
    let path = root.join("db/explain.sql");
    let text = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        crate::codebase::postgres::sql_file_offset_uses(&text),
        [
            (3, crate::codebase::postgres::OffsetUse::Other),
            (4, crate::codebase::postgres::OffsetUse::Zero),
            (5, crate::codebase::postgres::OffsetUse::Other),
            (7, crate::codebase::postgres::OffsetUse::Other),
        ]
    );
}

#[test]
fn static_append_keeps_fragment_lines_and_physical_suppressions() {
    let root = fixture("review-followups");
    let path = root.join("src/append-positions.ts");
    let mut findings = check_with_files(
        &root,
        &config_with_options("{}"),
        std::slice::from_ref(&path),
    )
    .unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [5, 8, 12, 18, 18, 21, 28]
    );
    let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [5, 8, 18, 18, 21]
    );
}
